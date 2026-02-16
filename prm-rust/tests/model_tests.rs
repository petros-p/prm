use prm::model::*;

// ==========================================================================
// ID TESTS
// ==========================================================================

#[test]
fn id_generate_creates_unique_ids() {
    let id1 = Id::<Person>::generate();
    let id2 = Id::<Person>::generate();
    assert_ne!(id1, id2);
}

#[test]
fn id_is_type_safe() {
    let person_id = Id::<Person>::generate();
    let circle_id = Id::<Circle>::generate();
    // They're different types at compile time, but we can verify UUIDs differ
    assert_ne!(person_id.value, circle_id.value);
}

// ==========================================================================
// USER TESTS
// ==========================================================================

#[test]
fn user_create_generates_id() {
    let user = User::create("Petros".into(), "petros@example.com".into());
    assert_eq!(user.name, "Petros");
    assert_eq!(user.email, "petros@example.com");
}

// ==========================================================================
// CONTACT INFO TESTS
// ==========================================================================

#[test]
fn address_holds_structured_data() {
    let address = Address {
        street: "123 Main St".into(),
        city: "Putnam".into(),
        state: "CT".into(),
        zip: "06260".into(),
        country: "USA".into(),
    };
    assert_eq!(address.city, "Putnam");
    assert_eq!(address.state, "CT");
}

#[test]
fn custom_contact_type_create_generates_id() {
    let ct = CustomContactType::create("Discord".into());
    assert_eq!(ct.name, "Discord");
}

#[test]
fn contact_entry_phone() {
    let entry = ContactEntry::phone("555-1234".into(), Some("work".into()));
    assert_eq!(entry.contact_type, ContactType::Phone);
    assert_eq!(
        entry.value,
        ContactValue::StringValue {
            value: "555-1234".into()
        }
    );
    assert_eq!(entry.label, Some("work".into()));
}

#[test]
fn contact_entry_email() {
    let entry = ContactEntry::email("test@example.com".into(), None);
    assert_eq!(entry.contact_type, ContactType::Email);
    assert_eq!(
        entry.value,
        ContactValue::StringValue {
            value: "test@example.com".into()
        }
    );
    assert_eq!(entry.label, None);
}

#[test]
fn contact_entry_address() {
    let address = Address {
        street: "123 Main St".into(),
        city: "Putnam".into(),
        state: "CT".into(),
        zip: "06260".into(),
        country: "USA".into(),
    };
    let entry = ContactEntry::address(address.clone(), Some("home".into()));
    assert_eq!(entry.contact_type, ContactType::PhysicalAddress);
    assert_eq!(entry.value, ContactValue::AddressValue { value: address });
    assert_eq!(entry.label, Some("home".into()));
}

#[test]
fn contact_entry_custom() {
    let type_id = Id::<CustomContactType>::generate();
    let entry = ContactEntry::custom(type_id, "petros#1234".into(), Some("gaming".into()));
    assert_eq!(entry.contact_type, ContactType::Custom { type_id });
    assert_eq!(
        entry.value,
        ContactValue::StringValue {
            value: "petros#1234".into()
        }
    );
    assert_eq!(entry.label, Some("gaming".into()));
}

// ==========================================================================
// INTERACTION MEDIUM TESTS
// ==========================================================================

#[test]
fn interaction_medium_all_contains_all_mediums() {
    let all = InteractionMedium::ALL;
    assert!(all.contains(&InteractionMedium::InPerson));
    assert!(all.contains(&InteractionMedium::Text));
    assert!(all.contains(&InteractionMedium::PhoneCall));
    assert!(all.contains(&InteractionMedium::VideoCall));
    assert!(all.contains(&InteractionMedium::SocialMedia));
    assert_eq!(all.len(), 5);
}

#[test]
fn interaction_medium_display_names() {
    assert_eq!(InteractionMedium::InPerson.display_name(), "In Person");
    assert_eq!(InteractionMedium::PhoneCall.display_name(), "Phone Call");
    assert_eq!(InteractionMedium::VideoCall.display_name(), "Video Call");
    assert_eq!(InteractionMedium::SocialMedia.display_name(), "Social Media");
    assert_eq!(InteractionMedium::Text.display_name(), "Text");
}

#[test]
fn interaction_medium_db_roundtrip() {
    for medium in InteractionMedium::ALL {
        let s = medium.to_db_str();
        let parsed = InteractionMedium::from_db_str(s).unwrap();
        assert_eq!(*medium, parsed);
    }
}

// ==========================================================================
// INTERACTION TESTS
// ==========================================================================

#[test]
fn interaction_create_in_person_sets_both_locations() {
    let today = chrono::Local::now().date_naive();
    let interaction = Interaction::create_in_person(
        "Coffee shop".into(),
        vec!["farming".into(), "weather".into()],
        None,
        today,
    );
    assert_eq!(interaction.medium, InteractionMedium::InPerson);
    assert_eq!(interaction.my_location, "Coffee shop");
    assert_eq!(interaction.their_location, Some("Coffee shop".into()));
}

#[test]
fn interaction_create_remote_allows_different_locations() {
    let today = chrono::Local::now().date_naive();
    let interaction = Interaction::create_remote(
        InteractionMedium::VideoCall,
        "Home".into(),
        Some("Office".into()),
        vec!["project planning".into()],
        None,
        today,
    );
    assert_eq!(interaction.medium, InteractionMedium::VideoCall);
    assert_eq!(interaction.my_location, "Home");
    assert_eq!(interaction.their_location, Some("Office".into()));
}

#[test]
fn interaction_create_remote_allows_optional_their_location() {
    let today = chrono::Local::now().date_naive();
    let interaction = Interaction::create_remote(
        InteractionMedium::Text,
        "Home".into(),
        None,
        vec!["quick chat".into()],
        None,
        today,
    );
    assert_eq!(interaction.their_location, None);
}

#[test]
#[should_panic(expected = "Use create_in_person")]
fn interaction_create_remote_rejects_in_person() {
    let today = chrono::Local::now().date_naive();
    Interaction::create_remote(
        InteractionMedium::InPerson,
        "Somewhere".into(),
        None,
        vec!["topic".into()],
        None,
        today,
    );
}

// ==========================================================================
// PERSON TESTS
// ==========================================================================

#[test]
fn person_create_generates_id_and_sets_defaults() {
    let person = Person::create("Alice".into());
    assert_eq!(person.name, "Alice");
    assert_eq!(person.nickname, None);
    assert_eq!(person.how_we_met, None);
    assert_eq!(person.birthday, None);
    assert_eq!(person.notes, None);
    assert_eq!(person.location, None);
    assert!(!person.is_self);
    assert!(!person.archived);
}

#[test]
fn person_create_self_sets_is_self() {
    let self_person = Person::create_self("Petros".into());
    assert_eq!(self_person.name, "Petros");
    assert!(self_person.is_self);
    assert!(!self_person.archived);
}

#[test]
fn person_can_be_archived() {
    let mut person = Person::create("Alice".into());
    person.archived = true;
    assert!(person.archived);
    assert_eq!(person.name, "Alice");
}

// ==========================================================================
// RELATIONSHIP LABEL TESTS
// ==========================================================================

#[test]
fn relationship_label_create_generates_id() {
    let label = RelationshipLabel::create("farming buddy".into());
    assert_eq!(label.name, "farming buddy");
    assert!(!label.archived);
}

#[test]
fn relationship_label_defaults_contains_expected() {
    let defaults = RelationshipLabel::defaults();
    let names: Vec<&str> = defaults.iter().map(|l| l.name.as_str()).collect();
    assert!(names.contains(&"me"));
    assert!(names.contains(&"friend"));
    assert!(names.contains(&"family"));
    assert!(names.contains(&"coworker"));
    assert!(names.contains(&"mentor"));

    for label in &defaults {
        assert!(!label.archived);
    }
}

// ==========================================================================
// RELATIONSHIP TESTS
// ==========================================================================

#[test]
fn relationship_create_starts_empty() {
    let person_id = Id::<Person>::generate();
    let rel = Relationship::create(person_id);
    assert_eq!(rel.person_id, person_id);
    assert!(rel.labels.is_empty());
    assert_eq!(rel.reminder_days, None);
}

// ==========================================================================
// CIRCLE TESTS
// ==========================================================================

#[test]
fn circle_create_starts_empty_and_not_archived() {
    let circle = Circle::create("Farming community".into(), None);
    assert_eq!(circle.name, "Farming community");
    assert_eq!(circle.description, None);
    assert!(circle.member_ids.is_empty());
    assert!(!circle.archived);
}

#[test]
fn circle_can_be_archived() {
    let mut circle = Circle::create("Test circle".into(), None);
    circle.archived = true;
    assert!(circle.archived);
    assert_eq!(circle.name, "Test circle");
}
