use jiff::Timestamp;

/// Maximum number of active families returned in one page.
pub const ACTIVE_PAGE_SIZE: usize = 20;

/// A family record.
///
/// Soft deletes are represented by [`Family::deleted_at`]: a record is active
/// while that column is `None`, and restoring it only clears the column. The
/// row is never physically removed, so history stays inspectable.
#[derive(Debug, toasty::Model)]
pub struct Family {
    /// Surrogate primary key, assigned by the database.
    #[key]
    #[auto]
    pub id: i64,

    /// Display name. Not unique: duplicate names are allowed.
    pub name: String,

    /// Optional free-form description.
    pub summary: Option<String>,

    /// When the record was soft-deleted, or `None` while it is active.
    ///
    /// Indexed because every list query filters on it.
    #[index]
    pub deleted_at: Option<Timestamp>,

    /// Optimistic concurrency token.
    ///
    /// Toasty sets this to `1` on create and increments it on every update. A
    /// stale write is rejected instead of silently overwriting, so a form must
    /// carry the version it was rendered with.
    #[version]
    pub version: u64,

    /// Set once, when the record is first inserted.
    #[auto]
    pub created_at: Timestamp,

    /// Refreshed on every create and update.
    #[auto]
    pub updated_at: Timestamp,
}

impl Family {
    /// Create a family. Toasty initializes its version and timestamps.
    pub async fn create_record(
        db: &mut toasty::Db,
        name: impl Into<String>,
        summary: Option<String>,
    ) -> toasty::Result<Self> {
        toasty::create!(Family {
            name: name.into(),
            summary,
        })
        .exec(db)
        .await
    }

    /// Find a family by ID unless it has been soft-deleted.
    pub async fn find_active(db: &mut toasty::Db, id: i64) -> toasty::Result<Option<Self>> {
        Self::filter_by_id(id)
            .filter(Self::fields().deleted_at().is_none())
            .first()
            .exec(db)
            .await
    }

    /// Find a soft-deleted family by ID so it can be restored.
    pub async fn find_deleted(db: &mut toasty::Db, id: i64) -> toasty::Result<Option<Self>> {
        Self::filter_by_id(id)
            .filter(Self::fields().deleted_at().is_some())
            .first()
            .exec(db)
            .await
    }

    /// Update editable fields, incrementing the optimistic-lock version.
    pub async fn update_details(
        &mut self,
        db: &mut toasty::Db,
        name: impl Into<String>,
        summary: Option<String>,
    ) -> toasty::Result<()> {
        self.update()
            .name(name.into())
            .summary(summary)
            .exec(db)
            .await?;

        Ok(())
    }

    /// Soft-delete the family while preserving the row for restoration.
    pub async fn soft_delete(&mut self, db: &mut toasty::Db) -> toasty::Result<()> {
        self.update()
            .deleted_at(Some(Timestamp::now()))
            .exec(db)
            .await?;

        Ok(())
    }

    /// Restore a soft-deleted family.
    pub async fn restore(&mut self, db: &mut toasty::Db) -> toasty::Result<()> {
        self.update().deleted_at(None).exec(db).await?;

        Ok(())
    }

    /// Fetch the first page of active families, newest first.
    ///
    /// Families are ordered by their auto-incrementing ID, which gives the
    /// cursor a unique and stable sort key. Active pages exclude soft-deleted
    /// records and contain at most [`ACTIVE_PAGE_SIZE`] rows.
    pub async fn first_active_page(
        db: &mut toasty::Db,
    ) -> toasty::Result<toasty::stmt::Page<Self>> {
        Self::filter(Self::fields().deleted_at().is_none())
            .order_by(Self::fields().id().desc())
            .paginate(ACTIVE_PAGE_SIZE)
            .exec(db)
            .await
    }

    /// Fetch the next active page after the family with `cursor_id`.
    pub async fn active_page_after(
        db: &mut toasty::Db,
        cursor_id: i64,
    ) -> toasty::Result<toasty::stmt::Page<Self>> {
        Self::filter(Self::fields().deleted_at().is_none())
            .order_by(Self::fields().id().desc())
            .paginate(ACTIVE_PAGE_SIZE)
            .after(cursor_id)
            .exec(db)
            .await
    }

    /// Fetch the previous active page before the family with `cursor_id`.
    pub async fn active_page_before(
        db: &mut toasty::Db,
        cursor_id: i64,
    ) -> toasty::Result<toasty::stmt::Page<Self>> {
        Self::filter(Self::fields().deleted_at().is_none())
            .order_by(Self::fields().id().desc())
            .paginate(ACTIVE_PAGE_SIZE)
            .before(cursor_id)
            .exec(db)
            .await
    }

    /// Fetch the first page of soft-deleted families, newest first.
    pub async fn first_deleted_page(
        db: &mut toasty::Db,
    ) -> toasty::Result<toasty::stmt::Page<Self>> {
        Self::filter(Self::fields().deleted_at().is_some())
            .order_by(Self::fields().id().desc())
            .paginate(ACTIVE_PAGE_SIZE)
            .exec(db)
            .await
    }

    /// Fetch the next deleted page after the family with `cursor_id`.
    pub async fn deleted_page_after(
        db: &mut toasty::Db,
        cursor_id: i64,
    ) -> toasty::Result<toasty::stmt::Page<Self>> {
        Self::filter(Self::fields().deleted_at().is_some())
            .order_by(Self::fields().id().desc())
            .paginate(ACTIVE_PAGE_SIZE)
            .after(cursor_id)
            .exec(db)
            .await
    }

    /// Fetch the previous deleted page before the family with `cursor_id`.
    pub async fn deleted_page_before(
        db: &mut toasty::Db,
        cursor_id: i64,
    ) -> toasty::Result<toasty::stmt::Page<Self>> {
        Self::filter(Self::fields().deleted_at().is_some())
            .order_by(Self::fields().id().desc())
            .paginate(ACTIVE_PAGE_SIZE)
            .before(cursor_id)
            .exec(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An isolated in-memory database with the current model schema.
    async fn memory_db() -> toasty::Db {
        let db = toasty::Db::builder()
            .models(toasty::models!(crate::*))
            .connect("sqlite::memory:")
            .await
            .expect("failed to connect");

        db.push_schema().await.expect("failed to create schema");

        db
    }

    #[tokio::test]
    async fn create_starts_at_version_one() {
        let mut db = memory_db().await;

        let family = Family::create_record(
            &mut db,
            "The Simpsons",
            Some(String::from("The Springfield family")),
        )
        .await
        .expect("create failed");

        assert_eq!(family.version, 1);
        assert!(family.deleted_at.is_none());
    }

    #[tokio::test]
    async fn summary_is_optional() {
        let mut db = memory_db().await;

        let created = Family::create_record(&mut db, "The Bikers", None)
            .await
            .expect("create failed");

        assert_eq!(created.summary, None);

        let loaded = Family::get_by_id(&mut db, &created.id)
            .await
            .expect("reload failed");

        assert_eq!(loaded.summary, None);
    }

    #[tokio::test]
    async fn stale_update_is_rejected() {
        let mut db = memory_db().await;

        let mut family = Family::create_record(&mut db, "Original", None)
            .await
            .expect("create failed");
        let mut stale = Family::get_by_id(&mut db, &family.id)
            .await
            .expect("load failed");

        family
            .update_details(&mut db, "Updated by someone else", None)
            .await
            .expect("first update failed");

        assert_eq!(family.version, 2);

        let result = stale
            .update_details(&mut db, "Should not be applied", None)
            .await;

        assert!(result.is_err(), "a stale write must be rejected");

        let reloaded = Family::get_by_id(&mut db, &family.id)
            .await
            .expect("reload failed");

        assert_eq!(reloaded.name, "Updated by someone else");
    }

    #[tokio::test]
    async fn cursor_pages_are_bounded_and_navigable() {
        let mut db = memory_db().await;

        for index in 0..(ACTIVE_PAGE_SIZE + 1) {
            Family::create_record(&mut db, format!("Family {index}"), None)
                .await
                .expect("create failed");
        }

        let first = Family::first_active_page(&mut db)
            .await
            .expect("first page failed");

        assert_eq!(first.len(), ACTIVE_PAGE_SIZE);
        assert!(first.has_next());
        assert!(!first.has_prev());

        let boundary = first.last().expect("first page should have rows").id;
        let second = Family::active_page_after(&mut db, boundary)
            .await
            .expect("next page failed");

        assert_eq!(second.len(), 1);
        assert!(!second.has_next());
        assert!(second.has_prev());

        let previous_boundary = second.first().expect("second page should have a row").id;
        let previous = Family::active_page_before(&mut db, previous_boundary)
            .await
            .expect("previous page failed");

        assert_eq!(previous.len(), ACTIVE_PAGE_SIZE);
        assert_eq!(
            previous.first().map(|family| family.id),
            first.first().map(|family| family.id)
        );
    }

    #[tokio::test]
    async fn active_pages_exclude_soft_deleted_records() {
        let mut db = memory_db().await;

        for index in 0..ACTIVE_PAGE_SIZE {
            Family::create_record(&mut db, format!("Family {index}"), None)
                .await
                .expect("create failed");
        }

        let mut removed = Family::create_record(&mut db, "Removed", None)
            .await
            .expect("create failed");
        removed
            .soft_delete(&mut db)
            .await
            .expect("soft delete failed");

        let page = Family::first_active_page(&mut db)
            .await
            .expect("active page failed");

        assert_eq!(page.len(), ACTIVE_PAGE_SIZE);
        assert!(page.iter().all(|family| family.deleted_at.is_none()));

        let total = Family::all().exec(&mut db).await.expect("count failed");
        assert_eq!(total.len(), ACTIVE_PAGE_SIZE + 1);
    }

    #[tokio::test]
    async fn soft_deleted_family_can_be_restored() {
        let mut db = memory_db().await;
        let mut family = Family::create_record(&mut db, "The Bikers", None)
            .await
            .expect("create failed");

        family
            .soft_delete(&mut db)
            .await
            .expect("soft delete failed");
        assert_eq!(family.version, 2);
        assert!(family.deleted_at.is_some());
        assert!(
            Family::find_active(&mut db, family.id)
                .await
                .expect("active lookup failed")
                .is_none()
        );

        let mut deleted = Family::find_deleted(&mut db, family.id)
            .await
            .expect("deleted lookup failed")
            .expect("soft-deleted family should still exist");
        deleted.restore(&mut db).await.expect("restore failed");

        assert_eq!(deleted.version, 3);
        assert!(deleted.deleted_at.is_none());
        assert!(
            Family::find_active(&mut db, family.id)
                .await
                .expect("active lookup failed")
                .is_some()
        );
        assert_eq!(
            Family::all()
                .exec(&mut db)
                .await
                .expect("list failed")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn deleted_pages_are_bounded_and_exclude_active_families() {
        let mut db = memory_db().await;
        let active = Family::create_record(&mut db, "Active", None)
            .await
            .expect("create failed");

        for index in 0..=ACTIVE_PAGE_SIZE {
            let mut family = Family::create_record(&mut db, format!("Deleted {index}"), None)
                .await
                .expect("create failed");
            family
                .soft_delete(&mut db)
                .await
                .expect("soft delete failed");
        }

        let first = Family::first_deleted_page(&mut db)
            .await
            .expect("first deleted page failed");

        assert_eq!(first.len(), ACTIVE_PAGE_SIZE);
        assert!(first.has_next());
        assert!(!first.has_prev());
        assert!(first.iter().all(|family| family.deleted_at.is_some()));

        let boundary = first.last().expect("first page should have rows").id;
        let second = Family::deleted_page_after(&mut db, boundary)
            .await
            .expect("next deleted page failed");

        assert_eq!(second.len(), 1);
        assert!(!second.has_next());
        assert!(second.has_prev());
        assert_eq!(
            Family::find_active(&mut db, active.id)
                .await
                .expect("active lookup failed")
                .map(|family| family.id),
            Some(active.id)
        );
    }
}
