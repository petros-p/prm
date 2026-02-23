use chrono::NaiveDate;
use prm::db::*;
use prm::model::*;
use prm::ops::*;
use prm::queries::*;

fn setup() -> (rusqlite::Connection, User, Person) {
    let conn = schema::test_connection();
    let user = User::create("Petros".into(), "petros@example.com".into());
    network_repo::insert_user(&conn, &user).unwrap();

    let self_person = Person::create_self("Petros".into());
    person_repo::insert(&conn, user.id, &self_person).unwrap();
    network_repo::set_network_metadata(&conn, user.id, self_person.id).unwrap();

    for label in RelationshipLabel::defaults() {
        relationship_repo::insert_label(&conn, user.id, &label).unwrap();
    }

    (conn, user, self_person)
}

// ==========================================================================
// PERSON QUERIES
// ==========================================================================

#[test]
fn active_people_excludes_archived() {
    let (conn, user, _) = setup();
    person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let bob = person_ops::add_person(&conn, user.id, "Bob", None, None, None, None, None).unwrap();
    person_ops::archive_person(&conn, bob.id).unwrap();

    let active = person_queries::active_people(&conn, user.id).unwrap();
    // Self + Alice
    assert_eq!(active.len(), 2);
    assert!(active.iter().all(|p| !p.archived));
}

#[test]
fn archived_people_only() {
    let (conn, user, _) = setup();
    let bob = person_ops::add_person(&conn, user.id, "Bob", None, None, None, None, None).unwrap();
    person_ops::archive_person(&conn, bob.id).unwrap();

    let archived = person_queries::archived_people(&conn, user.id).unwrap();
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].name, "Bob");
}

#[test]
fn find_person_by_name_query() {
    let (conn, user, _) = setup();
    person_ops::add_person(&conn, user.id, "Alice Smith", None, None, None, None, None).unwrap();
    person_ops::add_person(&conn, user.id, "Bob Jones", None, None, None, None, None).unwrap();

    let results = person_queries::find_by_name(&conn, user.id, "alice").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Alice Smith");
}

// ==========================================================================
// RELATIONSHIP QUERIES
// ==========================================================================

#[test]
fn people_with_label_query() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let _bob = person_ops::add_person(&conn, user.id, "Bob", None, None, None, None, None).unwrap();

    let labels = relationship_repo::find_active_labels(&conn, user.id).unwrap();
    let friend_id = labels.iter().find(|l| l.name == "friend").unwrap().id;

    relationship_ops::set_labels(&conn, user.id, alice.id, vec![friend_id]).unwrap();

    let friends = relationship_queries::people_with_label(&conn, user.id, friend_id).unwrap();
    assert_eq!(friends.len(), 1);
    assert_eq!(friends[0].name, "Alice");
}

#[test]
fn labels_for_person() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();

    let labels = relationship_repo::find_active_labels(&conn, user.id).unwrap();
    let friend_id = labels.iter().find(|l| l.name == "friend").unwrap().id;
    let family_id = labels.iter().find(|l| l.name == "family").unwrap().id;

    relationship_ops::set_labels(&conn, user.id, alice.id, vec![friend_id, family_id]).unwrap();

    let person_labels = relationship_queries::labels_for(&conn, alice.id).unwrap();
    assert_eq!(person_labels.len(), 2);
    let names: Vec<&str> = person_labels.iter().map(|l| l.name.as_str()).collect();
    assert!(names.contains(&"friend"));
    assert!(names.contains(&"family"));
}

// ==========================================================================
// INTERACTION QUERIES
// ==========================================================================

#[test]
fn days_since_interaction_query() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();

    let date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    interaction_ops::log_in_person(
        &conn,
        user.id,
        alice.id,
        "Coffee shop",
        vec!["chat".into()],
        None,
        date,
    )
    .unwrap();

    let as_of = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let days = interaction_queries::days_since_interaction(&conn, alice.id, as_of)
        .unwrap()
        .unwrap();
    assert_eq!(days, 14);
}

#[test]
fn interactions_in_date_range() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();

    let d1 = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
    let d2 = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let d3 = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();

    interaction_ops::log_in_person(&conn, user.id, alice.id, "Park", vec!["walk".into()], None, d1).unwrap();
    interaction_ops::log_in_person(&conn, user.id, alice.id, "Cafe", vec!["lunch".into()], None, d2).unwrap();
    interaction_ops::log_in_person(&conn, user.id, alice.id, "Beach", vec!["swim".into()], None, d3).unwrap();

    let from = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let to = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

    let results = interaction_queries::interactions_in_range(&conn, user.id, from, to).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.my_location, "Cafe");
}

// ==========================================================================
// CIRCLE QUERIES
// ==========================================================================

#[test]
fn circles_for_person_query() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();

    let _circle1 = circle_ops::create_circle(&conn, user.id, "Friends", None, vec![alice.id]).unwrap();
    let _circle2 = circle_ops::create_circle(&conn, user.id, "Coworkers", None, vec![]).unwrap();

    let circles = circle_queries::circles_for_person(&conn, user.id, alice.id).unwrap();
    assert_eq!(circles.len(), 1);
    assert_eq!(circles[0].name, "Friends");
}

// ==========================================================================
// REMINDER QUERIES
// ==========================================================================

#[test]
fn reminder_status_overdue() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();

    // Set reminder to 7 days
    relationship_ops::set_reminder(&conn, alice.id, Some(7)).unwrap();

    // Log interaction 10 days ago
    let interaction_date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    interaction_ops::log_in_person(
        &conn,
        user.id,
        alice.id,
        "Park",
        vec!["walk".into()],
        None,
        interaction_date,
    )
    .unwrap();

    let as_of = NaiveDate::from_ymd_opt(2024, 6, 11).unwrap();
    let status = reminder_queries::reminder_status(&conn, alice.id, as_of)
        .unwrap()
        .unwrap();

    assert_eq!(status.reminder_days, 7);
    assert_eq!(status.days_since_last_interaction, Some(10));

    match status.overdue_status {
        reminder_queries::OverdueStatus::DaysOverdue(days) => assert_eq!(days, 3),
        _ => panic!("Expected DaysOverdue"),
    }
}

#[test]
fn reminder_never_contacted() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    relationship_ops::set_reminder(&conn, alice.id, Some(7)).unwrap();

    let as_of = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let status = reminder_queries::reminder_status(&conn, alice.id, as_of)
        .unwrap()
        .unwrap();

    match status.overdue_status {
        reminder_queries::OverdueStatus::NeverContacted => {}
        _ => panic!("Expected NeverContacted"),
    }
}

#[test]
fn people_needing_reminder_query() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    let bob = person_ops::add_person(&conn, user.id, "Bob", None, None, None, None, None).unwrap();

    relationship_ops::set_reminder(&conn, alice.id, Some(7)).unwrap();
    relationship_ops::set_reminder(&conn, bob.id, Some(7)).unwrap();

    // Alice: interacted 10 days ago (overdue by 3)
    let d = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    interaction_ops::log_in_person(&conn, user.id, alice.id, "Park", vec!["walk".into()], None, d).unwrap();

    // Bob: interacted 3 days ago (not overdue)
    let d2 = NaiveDate::from_ymd_opt(2024, 6, 8).unwrap();
    interaction_ops::log_in_person(&conn, user.id, bob.id, "Cafe", vec!["chat".into()], None, d2).unwrap();

    let as_of = NaiveDate::from_ymd_opt(2024, 6, 11).unwrap();
    let overdue = reminder_queries::people_needing_reminder(&conn, user.id, as_of).unwrap();

    assert_eq!(overdue.len(), 1);
    assert_eq!(overdue[0].person.name, "Alice");
}

// ==========================================================================
// STATS QUERIES
// ==========================================================================

#[test]
fn network_stats() {
    let (conn, user, _) = setup();
    let alice = person_ops::add_person(&conn, user.id, "Alice", None, None, None, None, None).unwrap();
    circle_ops::create_circle(&conn, user.id, "Friends", None, vec![alice.id]).unwrap();

    let stats = stats_queries::stats(&conn, user.id).unwrap();
    assert_eq!(stats.total_people, 2); // self + alice
    assert_eq!(stats.active_people, 2);
    assert_eq!(stats.archived_people, 0);
    assert_eq!(stats.total_circles, 1);
    assert_eq!(stats.active_circles, 1);
}
