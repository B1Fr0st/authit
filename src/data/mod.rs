// Data access layer - handles retrieval and storage of application data
//
// This layer provides data operations that aren't direct entity CRUD:
// - Login logs / audit trails
// - Analytics data
// - Monitoring metrics
// - Report generation
//
// Unlike the db layer (which manages core entities like License, Product, Session),
// the data layer handles supplementary information and logs.

pub mod login;

pub use login::LoginData;
