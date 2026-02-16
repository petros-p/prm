use chrono::NaiveDate;
use prm::db::*;
use prm::model::*;

fn setup() -> (rusqlite::Connection, User, Person) {
    let conn = schema::test_connection();
    let user = User::create("Petros".into(), "petros@example.com".into());
    network_repo::insert_user(&conn, &user).unwrap();

    let self_person = Person::create_self("Petros".into());
    person_repo::insert(&conn, user.id, &self_person).unwrap();
    network_repo::set_network_metadata(&conn, user.id, self_person.id).unwrap();

    (conn, user, self_person)
}

// ==========================================================================
// PERSON REPO TESTS
// ==========================================================================

#[test]
fn person_insert_and_find() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let found = person_repo::find_by_id(&conn, alice.id).unwrap().unwrap();
    assert_eq!(found.name, "Alice");
    assert!(!found.is_self);
    assert!(!found.archived);
}

#[test]
fn person_update() {
    let (conn, user, _) = setup();

    let mut alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    alice.nickname = Some("Ali".into());
    alice.location = Some("NYC".into());
    person_repo::update(&conn, &alice).unwrap();

    let found = person_repo::find_by_id(&conn, alice.id).unwrap().unwrap();
    assert_eq!(found.nickname, Some("Ali".into()));
    assert_eq!(found.location, Some("NYC".into()));
}

#[test]
fn person_find_active_and_archived() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let mut bob = Person::create("Bob".into());
    bob.archived = true;
    person_repo::insert(&conn, user.id, &bob).unwrap();

    let active = person_repo::find_active_by_owner(&conn, user.id).unwrap();
    let archived = person_repo::find_archived_by_owner(&conn, user.id).unwrap();

    // Active includes self + Alice
    assert_eq!(active.len(), 2);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].name, "Bob");
}

#[test]
fn person_find_by_name() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice Smith".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let mut bob = Person::create("Bob Jones".into());
    bob.nickname = Some("Alice's friend".into());
    person_repo::insert(&conn, user.id, &bob).unwrap();

    let results = person_repo::find_by_name(&conn, user.id, "alice").unwrap();
    assert_eq!(results.len(), 2); // Alice Smith + Bob (nickname match)
}

#[test]
fn person_find_self() {
    let (conn, user, self_person) = setup();
    let found = person_repo::find_self(&conn, user.id).unwrap().unwrap();
    assert_eq!(found.id, self_person.id);
    assert!(found.is_self);
}

#[test]
fn person_birthday_roundtrip() {
    let (conn, user, _) = setup();

    let mut alice = Person::create("Alice".into());
    alice.birthday = Some(NaiveDate::from_ymd_opt(1990, 5, 15).unwrap());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let found = person_repo::find_by_id(&conn, alice.id).unwrap().unwrap();
    assert_eq!(
        found.birthday,
        Some(NaiveDate::from_ymd_opt(1990, 5, 15).unwrap())
    );
}

// ==========================================================================
// CONTACT REPO TESTS
// ==========================================================================

#[test]
fn contact_insert_and_find_phone() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let entry = ContactEntry::phone("555-1234".into(), Some("work".into()));
    contact_repo::insert(&conn, alice.id, &entry).unwrap();

    let contacts = contact_repo::find_by_person(&conn, alice.id).unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].contact_type, ContactType::Phone);
    assert_eq!(
        contacts[0].value,
        ContactValue::StringValue {
            value: "555-1234".into()
        }
    );
    assert_eq!(contacts[0].label, Some("work".into()));
}

#[test]
fn contact_insert_and_find_address() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let address = Address {
        street: "123 Main St".into(),
        city: "Putnam".into(),
        state: "CT".into(),
        zip: "06260".into(),
        country: "USA".into(),
    };
    let entry = ContactEntry::address(address.clone(), Some("home".into()));
    contact_repo::insert(&conn, alice.id, &entry).unwrap();

    let contacts = contact_repo::find_by_person(&conn, alice.id).unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].contact_type, ContactType::PhysicalAddress);
    assert_eq!(
        contacts[0].value,
        ContactValue::AddressValue { value: address }
    );
}

#[test]
fn contact_delete() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let entry = ContactEntry::phone("555-1234".into(), None);
    let entry_id = entry.id;
    contact_repo::insert(&conn, alice.id, &entry).unwrap();
    contact_repo::delete(&conn, entry_id).unwrap();

    let contacts = contact_repo::find_by_person(&conn, alice.id).unwrap();
    assert!(contacts.is_empty());
}

#[test]
fn custom_contact_type_roundtrip() {
    let (conn, user, _) = setup();

    let ct = CustomContactType::create("Discord".into());
    contact_repo::insert_custom_type(&conn, user.id, &ct).unwrap();

    let types = contact_repo::find_custom_types(&conn, user.id).unwrap();
    assert_eq!(types.len(), 1);
    assert_eq!(types[0].name, "Discord");

    let found = contact_repo::find_custom_type_by_name(&conn, user.id, "discord")
        .unwrap()
        .unwrap();
    assert_eq!(found.name, "Discord");
}

// ==========================================================================
// RELATIONSHIP REPO TESTS
// ==========================================================================

#[test]
fn relationship_upsert_and_find() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let label = RelationshipLabel::create("friend".into());
    relationship_repo::insert_label(&conn, user.id, &label).unwrap();

    let rel = Relationship {
        person_id: alice.id,
        labels: vec![label.id],
        reminder_days: Some(14),
    };
    relationship_repo::upsert(&conn, user.id, &rel).unwrap();

    let found = relationship_repo::find_by_person(&conn, alice.id)
        .unwrap()
        .unwrap();
    assert_eq!(found.person_id, alice.id);
    assert_eq!(found.labels.len(), 1);
    assert_eq!(found.labels[0], label.id);
    assert_eq!(found.reminder_days, Some(14));
}

#[test]
fn relationship_update_reminder() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let rel = Relationship::create(alice.id);
    relationship_repo::upsert(&conn, user.id, &rel).unwrap();

    relationship_repo::update_reminder(&conn, alice.id, Some(7)).unwrap();

    let found = relationship_repo::find_by_person(&conn, alice.id)
        .unwrap()
        .unwrap();
    assert_eq!(found.reminder_days, Some(7));
}

#[test]
fn label_crud() {
    let (conn, user, _) = setup();

    let label = RelationshipLabel::create("friend".into());
    relationship_repo::insert_label(&conn, user.id, &label).unwrap();

    let found = relationship_repo::find_label_by_id(&conn, label.id)
        .unwrap()
        .unwrap();
    assert_eq!(found.name, "friend");
    assert!(!found.archived);

    // Update
    let mut updated = found;
    updated.name = "close friend".into();
    relationship_repo::update_label_row(&conn, &updated).unwrap();

    let found2 = relationship_repo::find_label_by_id(&conn, label.id)
        .unwrap()
        .unwrap();
    assert_eq!(found2.name, "close friend");
}

// ==========================================================================
// INTERACTION REPO TESTS
// ==========================================================================

#[test]
fn interaction_insert_and_find() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let rel = Relationship::create(alice.id);
    relationship_repo::upsert(&conn, user.id, &rel).unwrap();

    let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let interaction = Interaction::create_in_person(
        "Coffee shop".into(),
        vec!["farming".into(), "weather".into()],
        Some("Great chat".into()),
        date,
    );
    interaction_repo::insert(&conn, alice.id, &interaction).unwrap();

    let found = interaction_repo::find_by_person(&conn, alice.id).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].my_location, "Coffee shop");
    assert_eq!(found[0].medium, InteractionMedium::InPerson);
    assert_eq!(found[0].note, Some("Great chat".into()));
    assert!(found[0].topics.contains(&"farming".to_string()));
    assert!(found[0].topics.contains(&"weather".to_string()));
}

#[test]
fn interaction_last_date() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let rel = Relationship::create(alice.id);
    relationship_repo::upsert(&conn, user.id, &rel).unwrap();

    let date1 = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

    let i1 = Interaction::create_in_person("Park".into(), vec!["walk".into()], None, date1);
    let i2 = Interaction::create_in_person("Cafe".into(), vec!["lunch".into()], None, date2);

    interaction_repo::insert(&conn, alice.id, &i1).unwrap();
    interaction_repo::insert(&conn, alice.id, &i2).unwrap();

    let last = interaction_repo::find_last_interaction_date(&conn, alice.id)
        .unwrap()
        .unwrap();
    assert_eq!(last, date2);
}

// ==========================================================================
// CIRCLE REPO TESTS
// ==========================================================================

#[test]
fn circle_insert_and_find() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();

    let mut circle = Circle::create("Friends".into(), Some("Close friends".into()));
    circle.member_ids = vec![alice.id];
    circle_repo::insert(&conn, user.id, &circle).unwrap();

    let found = circle_repo::find_by_id(&conn, circle.id).unwrap().unwrap();
    assert_eq!(found.name, "Friends");
    assert_eq!(found.description, Some("Close friends".into()));
    assert_eq!(found.member_ids.len(), 1);
    assert!(found.member_ids.contains(&alice.id));
}

#[test]
fn circle_add_remove_members() {
    let (conn, user, _) = setup();

    let alice = Person::create("Alice".into());
    let bob = Person::create("Bob".into());
    person_repo::insert(&conn, user.id, &alice).unwrap();
    person_repo::insert(&conn, user.id, &bob).unwrap();

    let circle = Circle::create("Friends".into(), None);
    circle_repo::insert(&conn, user.id, &circle).unwrap();

    circle_repo::add_members(&conn, circle.id, &[alice.id, bob.id]).unwrap();

    let found = circle_repo::find_by_id(&conn, circle.id).unwrap().unwrap();
    assert_eq!(found.member_ids.len(), 2);

    circle_repo::remove_members(&conn, circle.id, &[bob.id]).unwrap();

    let found2 = circle_repo::find_by_id(&conn, circle.id).unwrap().unwrap();
    assert_eq!(found2.member_ids.len(), 1);
    assert!(found2.member_ids.contains(&alice.id));
}

// ==========================================================================
// NETWORK REPO TESTS
// ==========================================================================

#[test]
fn network_metadata_roundtrip() {
    let (conn, user, self_person) = setup();

    let self_id = network_repo::get_self_id(&conn, user.id).unwrap().unwrap();
    assert_eq!(self_id, self_person.id);
}

#[test]
fn find_first_user() {
    let (conn, user, _) = setup();
    let found = network_repo::find_first_user(&conn).unwrap().unwrap();
    assert_eq!(found.id, user.id);
}
