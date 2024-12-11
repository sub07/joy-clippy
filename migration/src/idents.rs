use sea_orm_migration::sea_orm::DeriveIden;

#[derive(DeriveIden)]
pub enum I {
    Id,
    Entry,
    Data,
    AddedAt,
}
