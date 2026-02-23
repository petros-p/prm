use chrono::NaiveDate;
use prm::db::*;
use prm::model::*;
use prm::ops::*;

fn setup() -> (rusqlite::Connection, User, Person) {
    let conn = schema::test_connection();
    let user = User::create("Petros".into(), "petros@example.com".into());
    network_repo::insert_user(&conn, &user).unwrap();

    let self_person = Person::create_self("Petros".into());
    person_repo::insert(&conn, user.id, &self_person).unwrap();
    network_repo::set_network_metadata(&conn, user.id, self_person.id).unwrap();

    // Insert default labels
    for label in RelationshipLabel::defaults() {
        relationship_repo::insert_label(&conn, user.id, &label).unwrap();
    }

    (conn, user, self_person)
}

// ==========================================================================
// PERSON OPS TESTS
// ==========================================================================

#[test]
fn add_person_with_valid_name() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    assert_eq!(person.name, "Alice");
    assert!(!person.archived);
}

#[test]
fn add_person_trims_name() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "  Alice  ", None, None, None, None, None).unwrap();
    assert_eq!(person.name, "Alice");
}

#[test]
fn add_person_rejects_blank_name() {
    let (conn, user, _) = setup();
    let result = person_ops::add_person(&conn, user.id, "   ", None, None, None, None, None);
    assert!(result.is_err());
}

#[test]
fn add_person_with_all_fields() {
    let (conn, user, _) = setup();
    let birthday = NaiveDate::from_ymd_opt(1990, 5, 15).unwrap();
    let person = person_ops::add_person(
        &conn,
        user.id,
        "Bob",
        Some("Bobby"),
        Some("College roommate"),
        Some(birthday),
        Some("Loves hiking"),
        Some("Coffee shop downtown"),
    )
    .unwrap();

    assert_eq!(person.name, "Bob");
    assert_eq!(person.nickname, Some("Bobby".into()));
    assert_eq!(person.how_we_met, Some("College roommate".into()));
    assert_eq!(person.birthday, Some(birthday));
    assert_eq!(person.notes, Some("Loves hiking".into()));
    assert_eq!(person.location, Some("Coffee shop downtown".into()));
}

#[test]
fn archive_person_works() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let archived = person_ops::archive_person(&conn, person.id).unwrap();
    assert!(archived.archived);
}

#[test]
fn archive_self_fails() {
    let (conn, _, self_person) = setup();
    let result = person_ops::archive_person(&conn, self_person.id);
    assert!(result.is_err());
}

#[test]
fn unarchive_person_works() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    person_ops::archive_person(&conn, person.id).unwrap();
    let unarchived = person_ops::unarchive_person(&conn, person.id).unwrap();
    assert!(!unarchived.archived);
}

#[test]
fn update_person_name() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let updated = person_ops::update_person(&conn, person.id, Some("Alicia"), None, None, None, None, None).unwrap();
    assert_eq!(updated.name, "Alicia");
}

// ==========================================================================
// CONTACT OPS TESTS
// ==========================================================================

#[test]
fn add_phone_to_person() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let entry = contact_ops::add_phone(&conn, person.id, "555-1234", Some("work")).unwrap();
    assert_eq!(entry.contact_type, ContactType::Phone);

    let contacts = contact_repo::find_by_person(&conn, person.id).unwrap();
    assert_eq!(contacts.len(), 1);
}

#[test]
fn add_phone_rejects_blank() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let result = contact_ops::add_phone(&conn, person.id, "  ", None);
    assert!(result.is_err());
}

#[test]
fn create_custom_contact_type_and_use() {
    let (conn, user, _) = setup();
    let ct = contact_ops::create_custom_contact_type(&conn, user.id, "Discord").unwrap();
    assert_eq!(ct.name, "Discord");

    // Duplicate should fail
    let result = contact_ops::create_custom_contact_type(&conn, user.id, "discord");
    assert!(result.is_err());
}

// ==========================================================================
// RELATIONSHIP OPS TESTS
// ==========================================================================

#[test]
fn set_relationship_with_labels() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();

    let labels = relationship_repo::find_active_labels(&conn, user.id).unwrap();
    let friend_label = labels.iter().find(|l| l.name == "friend").unwrap();

    let rel = relationship_ops::set_relationship(
        &conn,
        user.id,
        person.id,
        vec![friend_label.id],
        Some(14),
    )
    .unwrap();

    assert_eq!(rel.labels.len(), 1);
    assert_eq!(rel.reminder_days, Some(14));
}

#[test]
fn set_reminder_rejects_zero() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let result = relationship_ops::set_reminder(&conn, person.id, Some(0));
    assert!(result.is_err());
}

// ==========================================================================
// INTERACTION OPS TESTS
// ==========================================================================

#[test]
fn log_in_person_interaction() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();

    let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let interaction = interaction_ops::log_in_person(
        &conn,
        user.id,
        person.id,
        "Coffee shop",
        vec!["farming".into(), "weather".into()],
        Some("Great chat"),
        date,
    )
    .unwrap();

    assert_eq!(interaction.medium, InteractionMedium::InPerson);
    assert_eq!(interaction.my_location, "Coffee shop");
    assert_eq!(interaction.their_location, Some("Coffee shop".into()));
}

#[test]
fn log_remote_interaction() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();

    let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let interaction = interaction_ops::log_remote(
        &conn,
        user.id,
        person.id,
        InteractionMedium::VideoCall,
        "Home",
        Some("Office"),
        vec!["project".into()],
        None,
        date,
    )
    .unwrap();

    assert_eq!(interaction.medium, InteractionMedium::VideoCall);
}

#[test]
fn log_interaction_rejects_blank_location() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let result = interaction_ops::log_in_person(
        &conn,
        user.id,
        person.id,
        "  ",
        vec!["topic".into()],
        None,
        date,
    );
    assert!(result.is_err());
}

#[test]
fn log_interaction_rejects_empty_topics() {
    let (conn, user, _) = setup();
    let person = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let result = interaction_ops::log_in_person(
        &conn,
        user.id,
        person.id,
        "Coffee shop",
        vec![],
        None,
        date,
    );
    assert!(result.is_err());
}

// ==========================================================================
// CIRCLE OPS TESTS
// ==========================================================================

#[test]
fn create_circle_works() {
    let (conn, user, _) = setup();
    let circle = circle_ops::create_circle(&conn, user.id, "Friends", Some("Close friends"), vec![]).unwrap();
    assert_eq!(circle.name, "Friends");
    assert_eq!(circle.description, Some("Close friends".into()));
}

#[test]
fn create_circle_rejects_blank_name() {
    let (conn, user, _) = setup();
    let result = circle_ops::create_circle(&conn, user.id, "  ", None, vec![]);
    assert!(result.is_err());
}

#[test]
fn archive_and_unarchive_circle() {
    let (conn, user, _) = setup();
    let circle = circle_ops::create_circle(&conn, user.id, "Friends", None, vec![]).unwrap();

    let archived = circle_ops::archive_circle(&conn, circle.id).unwrap();
    assert!(archived.archived);

    let unarchived = circle_ops::unarchive_circle(&conn, circle.id).unwrap();
    assert!(!unarchived.archived);
}

#[test]
fn add_and_remove_circle_members() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let bob = person_ops::add_person(&conn, user.id, "Bob", None, None, None, None, None).unwrap();
    let circle = circle_ops::create_circle(&conn, user.id, "Friends", None, vec![]).unwrap();

    let updated = circle_ops::add_members(&conn, circle.id, vec![alice.id, bob.id]).unwrap();
    assert_eq!(updated.member_ids.len(), 2);

    let updated2 = circle_ops::remove_members(&conn, circle.id, vec![bob.id]).unwrap();
    assert_eq!(updated2.member_ids.len(), 1);
}

// ==========================================================================
// LABEL OPS TESTS
// ==========================================================================

#[test]
fn add_label_works() {
    let (conn, user, _) = setup();
    let label = label_ops::add_label(&conn, user.id, "farming buddy").unwrap();
    assert_eq!(label.name, "farming buddy");
    assert!(!label.archived);
}

#[test]
fn add_duplicate_label_fails() {
    let (conn, user, _) = setup();
    // "friend" is already a default label
    let result = label_ops::add_label(&conn, user.id, "friend");
    assert!(result.is_err());
}

#[test]
fn archive_and_unarchive_label() {
    let (conn, user, _) = setup();
    let label = label_ops::add_label(&conn, user.id, "farming buddy").unwrap();

    let archived = label_ops::archive_label(&conn, label.id).unwrap();
    assert!(archived.archived);

    let unarchived = label_ops::unarchive_label(&conn, label.id).unwrap();
    assert!(!unarchived.archived);
}

#[test]
fn update_label_name() {
    let (conn, user, _) = setup();
    let label = label_ops::add_label(&conn, user.id, "farming buddy").unwrap();
    let updated = label_ops::update_label(&conn, user.id, label.id, Some("farm friend")).unwrap();
    assert_eq!(updated.name, "farm friend");
}
