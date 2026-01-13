package network

import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers
import java.time.LocalDate

class ModelSpec extends AnyFunSuite with Matchers {

  // ==========================================================================
  // ID TESTS
  // ==========================================================================

  test("Id.generate creates unique IDs") {
    val id1 = Id.generate[Person]
    val id2 = Id.generate[Person]
    
    id1 should not equal id2
  }

  test("Id is type-safe - Person ID and Circle ID are different types") {
    val personId: Id[Person] = Id.generate[Person]
    val circleId: Id[Circle] = Id.generate[Circle]
    
    personId.value should not equal circleId.value
  }

  // ==========================================================================
  // USER TESTS
  // ==========================================================================

  test("User.create generates ID") {
    val user = User.create("Petros", "petros@example.com")
    
    user.name shouldBe "Petros"
    user.email shouldBe "petros@example.com"
    user.id.value shouldBe a[java.util.UUID]
  }

  // ==========================================================================
  // CONTACT INFO TESTS
  // ==========================================================================

  test("Address holds structured data") {
    val address = Address(
      street = "123 Main St",
      city = "Putnam",
      state = "CT",
      zip = "06260",
      country = "USA"
    )
    
    address.city shouldBe "Putnam"
    address.state shouldBe "CT"
  }

  test("CustomContactType.create generates ID") {
    val contactType = CustomContactType.create("Discord")
    
    contactType.name shouldBe "Discord"
    contactType.id.value shouldBe a[java.util.UUID]
  }

  test("ContactEntry.phone creates phone entry") {
    val entry = ContactEntry.phone("555-1234", Some("work"))
    
    entry.contactType shouldBe ContactType.Phone
    entry.value shouldBe ContactValue.StringValue("555-1234")
    entry.label shouldBe Some("work")
  }

  test("ContactEntry.email creates email entry") {
    val entry = ContactEntry.email("test@example.com", None)
    
    entry.contactType shouldBe ContactType.Email
    entry.value shouldBe ContactValue.StringValue("test@example.com")
    entry.label shouldBe None
  }

  test("ContactEntry.address creates address entry") {
    val address = Address("123 Main St", "Putnam", "CT", "06260", "USA")
    val entry = ContactEntry.address(address, Some("home"))
    
    entry.contactType shouldBe ContactType.PhysicalAddress
    entry.value shouldBe ContactValue.AddressValue(address)
    entry.label shouldBe Some("home")
  }

  test("ContactEntry.custom creates custom entry") {
    val typeId = Id.generate[CustomContactType]
    val entry = ContactEntry.custom(typeId, "petros#1234", Some("gaming"))
    
    entry.contactType shouldBe ContactType.Custom(typeId)
    entry.value shouldBe ContactValue.StringValue("petros#1234")
    entry.label shouldBe Some("gaming")
  }

  // ==========================================================================
  // INTERACTION MEDIUM TESTS
  // ==========================================================================

  test("InteractionMedium.all contains all mediums") {
    InteractionMedium.all should contain allOf (
      InteractionMedium.InPerson,
      InteractionMedium.Text,
      InteractionMedium.PhoneCall,
      InteractionMedium.VideoCall,
      InteractionMedium.SocialMedia
    )
  }

  test("InteractionMedium.name returns human-readable names") {
    InteractionMedium.name(InteractionMedium.InPerson) shouldBe "In Person"
    InteractionMedium.name(InteractionMedium.PhoneCall) shouldBe "Phone Call"
    InteractionMedium.name(InteractionMedium.VideoCall) shouldBe "Video Call"
    InteractionMedium.name(InteractionMedium.SocialMedia) shouldBe "Social Media"
    InteractionMedium.name(InteractionMedium.Text) shouldBe "Text"
  }

  // ==========================================================================
  // INTERACTION TESTS
  // ==========================================================================

  test("Interaction.createInPerson sets both locations to same value") {
    val interaction = Interaction.createInPerson(
      location = "Coffee shop",
      topics = Set("farming", "weather")
    )
    
    interaction.medium shouldBe InteractionMedium.InPerson
    interaction.myLocation shouldBe "Coffee shop"
    interaction.theirLocation shouldBe Some("Coffee shop")
    interaction.date shouldBe LocalDate.now()
  }

  test("Interaction.createRemote allows different locations") {
    val interaction = Interaction.createRemote(
      medium = InteractionMedium.VideoCall,
      myLocation = "Home",
      theirLocation = Some("Office"),
      topics = Set("project planning")
    )
    
    interaction.medium shouldBe InteractionMedium.VideoCall
    interaction.myLocation shouldBe "Home"
    interaction.theirLocation shouldBe Some("Office")
  }

  test("Interaction.createRemote allows optional theirLocation") {
    val interaction = Interaction.createRemote(
      medium = InteractionMedium.Text,
      myLocation = "Home",
      theirLocation = None,
      topics = Set("quick chat")
    )
    
    interaction.theirLocation shouldBe None
  }

  test("Interaction.createRemote rejects InPerson medium") {
    an[IllegalArgumentException] should be thrownBy {
      Interaction.createRemote(
        medium = InteractionMedium.InPerson,
        myLocation = "Somewhere",
        theirLocation = None,
        topics = Set("topic")
      )
    }
  }

  // ==========================================================================
  // PERSON TESTS
  // ==========================================================================

  test("Person.create generates ID and sets defaults") {
    val person = Person.create("Alice")
    
    person.name shouldBe "Alice"
    person.nickname shouldBe None
    person.howWeMet shouldBe None
    person.birthday shouldBe None
    person.notes shouldBe None
    person.defaultLocation shouldBe None
    person.contactInfo shouldBe empty
    person.isSelf shouldBe false
    person.archived shouldBe false
  }

  test("Person.create accepts all optional fields including contactInfo") {
    val birthday = LocalDate.of(1990, 5, 15)
    val phone = ContactEntry.phone("555-1234")
    val person = Person.create(
      name = "Bob",
      nickname = Some("Bobby"),
      howWeMet = Some("College roommate"),
      birthday = Some(birthday),
      notes = Some("Loves hiking"),
      defaultLocation = Some("Coffee shop downtown"),
      contactInfo = List(phone)
    )
    
    person.name shouldBe "Bob"
    person.nickname shouldBe Some("Bobby")
    person.howWeMet shouldBe Some("College roommate")
    person.birthday shouldBe Some(birthday)
    person.notes shouldBe Some("Loves hiking")
    person.defaultLocation shouldBe Some("Coffee shop downtown")
    person.contactInfo should have size 1
    person.isSelf shouldBe false
  }

  test("Person.createSelf sets isSelf to true") {
    val self = Person.createSelf("Petros")
    
    self.name shouldBe "Petros"
    self.isSelf shouldBe true
    self.archived shouldBe false
  }

  test("Person can be archived via copy") {
    val person = Person.create("Alice")
    val archived = person.copy(archived = true)
    
    archived.archived shouldBe true
    archived.name shouldBe "Alice"
  }

  // ==========================================================================
  // RELATIONSHIP LABEL TESTS
  // ==========================================================================

  test("RelationshipLabel.create generates ID") {
    val label = RelationshipLabel.create("farming buddy")
    
    label.name shouldBe "farming buddy"
    label.id.value shouldBe a[java.util.UUID]
  }

  test("RelationshipLabel.defaults contains expected labels") {
    val defaultNames = RelationshipLabel.defaults.map(_.name)
    
    defaultNames should contain("me")
    defaultNames should contain("friend")
    defaultNames should contain("family")
    defaultNames should contain("coworker")
    defaultNames should contain("mentor")
  }

  // ==========================================================================
  // RELATIONSHIP TESTS
  // ==========================================================================

  test("Relationship.create starts with empty interaction history") {
    val personId = Id.generate[Person]
    val relationship = Relationship.create(personId)
    
    relationship.personId shouldBe personId
    relationship.labels shouldBe empty
    relationship.reminderDays shouldBe None
    relationship.interactionHistory shouldBe empty
  }

  // ==========================================================================
  // CIRCLE TESTS
  // ==========================================================================

  test("Circle.create starts with no members and not archived") {
    val circle = Circle.create("Farming community")
    
    circle.name shouldBe "Farming community"
    circle.description shouldBe None
    circle.memberIds shouldBe empty
    circle.archived shouldBe false
  }

  test("Circle can be archived via copy") {
    val circle = Circle.create("Test circle")
    val archived = circle.copy(archived = true)
    
    archived.archived shouldBe true
    archived.name shouldBe "Test circle"
  }

  // ==========================================================================
  // NETWORK TESTS
  // ==========================================================================

  test("Network.create initializes with owner, self, and default labels") {
    val owner = User.create("Petros", "petros@example.com")
    val self = Person.createSelf("Petros")
    val network = Network.create(owner, self)
    
    network.ownerId shouldBe owner.id
    network.selfId shouldBe self.id
    network.people should have size 1
    network.people(self.id) shouldBe self
    network.relationships shouldBe empty
    network.circles shouldBe empty
    network.relationshipLabels should have size RelationshipLabel.defaults.size
    network.customContactTypes shouldBe empty
  }

  test("Network.create requires isSelf to be true") {
    val owner = User.create("Petros", "petros@example.com")
    val notSelf = Person.create("Alice")
    
    an[IllegalArgumentException] should be thrownBy {
      Network.create(owner, notSelf)
    }
  }

  test("Person can be added to network via copy") {
    val owner = User.create("Petros", "petros@example.com")
    val self = Person.createSelf("Petros")
    val network = Network.create(owner, self)
    
    val alice = Person.create("Alice")
    val updated = network.copy(
      people = network.people + (alice.id -> alice)
    )
    
    updated.people should have size 2
    updated.people(alice.id) shouldBe alice
  }
}
