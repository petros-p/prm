package network

import java.time.LocalDate
import java.util.UUID

// ============================================================================
// IDENTIFIERS
// ============================================================================

/**
 * Unique identifier for entities in the system.
 * Wrapper around UUID for type safety and future flexibility.
 */
case class Id[A](value: UUID) extends AnyVal

object Id {
  def generate[A]: Id[A] = Id(UUID.randomUUID())
}

// ============================================================================
// USER (for future multi-user support)
// ============================================================================

/**
 * A user of the system. Each user has their own separate network.
 * 
 * Note: Authentication (passwords, sessions, etc.) is not yet implemented.
 * This is a stub for future multi-user support.
 *
 * @param id    Unique identifier
 * @param name  Display name
 * @param email Email address (will be used for login in the future)
 */
case class User(
  id: Id[User],
  name: String,
  email: String
)

object User {
  def create(name: String, email: String): User = User(
    id = Id.generate[User],
    name = name,
    email = email
  )
}

// ============================================================================
// CONTACT INFORMATION
// ============================================================================

/**
 * A structured physical address.
 */
case class Address(
  street: String,
  city: String,
  state: String,
  zip: String,
  country: String
)

/**
 * A user-defined contact type (e.g., "Discord", "LinkedIn", "WhatsApp").
 * These are defined at the network level and can be reused across people.
 *
 * @param id   Unique identifier
 * @param name The name of this contact type (e.g., "Discord")
 */
case class CustomContactType(
  id: Id[CustomContactType],
  name: String
)

object CustomContactType {
  def create(name: String): CustomContactType = CustomContactType(
    id = Id.generate[CustomContactType],
    name = name
  )
}

/**
 * The type of contact information.
 * Built-in types: Phone, Email, Address
 * Custom types reference a user-defined CustomContactType
 */
sealed trait ContactType

object ContactType {
  case object Phone extends ContactType
  case object Email extends ContactType
  case object PhysicalAddress extends ContactType
  case class Custom(typeId: Id[CustomContactType]) extends ContactType
}

/**
 * The value of a contact entry. 
 * Phone, Email, and Custom use string values.
 * PhysicalAddress uses a structured Address.
 */
sealed trait ContactValue

object ContactValue {
  case class StringValue(value: String) extends ContactValue
  case class AddressValue(value: Address) extends ContactValue
}

/**
 * A single contact entry for a person.
 *
 * @param id         Unique identifier
 * @param contactType The type of contact (Phone, Email, Address, or Custom)
 * @param value      The contact value
 * @param label      Optional label (e.g., "work", "personal", "parents' house")
 */
case class ContactEntry(
  id: Id[ContactEntry],
  contactType: ContactType,
  value: ContactValue,
  label: Option[String]
)

object ContactEntry {
  def phone(number: String, label: Option[String] = None): ContactEntry = ContactEntry(
    id = Id.generate[ContactEntry],
    contactType = ContactType.Phone,
    value = ContactValue.StringValue(number),
    label = label
  )

  def email(address: String, label: Option[String] = None): ContactEntry = ContactEntry(
    id = Id.generate[ContactEntry],
    contactType = ContactType.Email,
    value = ContactValue.StringValue(address),
    label = label
  )

  def address(address: Address, label: Option[String] = None): ContactEntry = ContactEntry(
    id = Id.generate[ContactEntry],
    contactType = ContactType.PhysicalAddress,
    value = ContactValue.AddressValue(address),
    label = label
  )

  def custom(typeId: Id[CustomContactType], value: String, label: Option[String] = None): ContactEntry = ContactEntry(
    id = Id.generate[ContactEntry],
    contactType = ContactType.Custom(typeId),
    value = ContactValue.StringValue(value),
    label = label
  )
}

// ============================================================================
// INTERACTION
// ============================================================================

/**
 * How the interaction took place.
 */
sealed trait InteractionMedium

object InteractionMedium {
  /** Both people were physically in the same location */
  case object InPerson extends InteractionMedium
  
  /** Text message (SMS, iMessage, etc.) */
  case object Text extends InteractionMedium
  
  /** Voice call */
  case object PhoneCall extends InteractionMedium
  
  /** Video call (Zoom, FaceTime, etc.) */
  case object VideoCall extends InteractionMedium
  
  /** Social media interaction (DM, comment, etc.) */
  case object SocialMedia extends InteractionMedium
  
  /** All available mediums for display/selection */
  val all: List[InteractionMedium] = List(InPerson, Text, PhoneCall, VideoCall, SocialMedia)
  
  def name(medium: InteractionMedium): String = medium match {
    case InPerson => "In Person"
    case Text => "Text"
    case PhoneCall => "Phone Call"
    case VideoCall => "Video Call"
    case SocialMedia => "Social Media"
  }
}

/**
 * A single interaction between you and another person.
 *
 * @param id            Unique identifier for this interaction
 * @param date          When the interaction occurred (defaults to today when created)
 * @param medium        How the interaction took place (In Person, Text, Phone Call, etc.)
 * @param myLocation    Where you were during the interaction (required)
 * @param theirLocation Where they were during the interaction.
 *                      For InPerson, this should equal myLocation.
 *                      For remote interactions, this is optional.
 * @param topics        What was discussed (at least one required)
 * @param note          Optional free-form notes about the interaction
 */
case class Interaction(
  id: Id[Interaction],
  date: LocalDate,
  medium: InteractionMedium,
  myLocation: String,
  theirLocation: Option[String],
  topics: Set[String],
  note: Option[String]
)

object Interaction {
  /**
   * Creates a new in-person interaction.
   * Both people are at the same location.
   */
  def createInPerson(
    location: String,
    topics: Set[String],
    note: Option[String] = None,
    date: LocalDate = LocalDate.now()
  ): Interaction = Interaction(
    id = Id.generate[Interaction],
    date = date,
    medium = InteractionMedium.InPerson,
    myLocation = location,
    theirLocation = Some(location),
    topics = topics,
    note = note
  )

  /**
   * Creates a new remote interaction (Text, Phone Call, Video Call, Social Media).
   */
  def createRemote(
    medium: InteractionMedium,
    myLocation: String,
    theirLocation: Option[String],
    topics: Set[String],
    note: Option[String] = None,
    date: LocalDate = LocalDate.now()
  ): Interaction = {
    require(medium != InteractionMedium.InPerson, "Use createInPerson for in-person interactions")
    Interaction(
      id = Id.generate[Interaction],
      date = date,
      medium = medium,
      myLocation = myLocation,
      theirLocation = theirLocation,
      topics = topics,
      note = note
    )
  }
}

// ============================================================================
// PERSON
// ============================================================================

/**
 * A person in your network (including yourself).
 *
 * @param id              Unique identifier
 * @param name            Full name
 * @param nickname        Optional nickname or preferred name
 * @param howWeMet        Optional description of how you met
 * @param birthday        Optional birthday
 * @param notes           Optional free-form notes about this person
 * @param location Location for interactions (convenience for logging)
 * @param contactInfo     List of contact entries (phone, email, address, custom)
 * @param isSelf          True if this person represents you (the network owner)
 * @param archived        True if this person is archived (hidden from main view)
 */
case class Person(
  id: Id[Person],
  name: String,
  nickname: Option[String],
  howWeMet: Option[String],
  birthday: Option[LocalDate],
  notes: Option[String],
  location: Option[String],
  contactInfo: List[ContactEntry],
  isSelf: Boolean,
  archived: Boolean
)

object Person {
  /**
   * Creates a new person with auto-generated ID.
   * Defaults to not-self and not-archived.
   */
  def create(
    name: String,
    nickname: Option[String] = None,
    howWeMet: Option[String] = None,
    birthday: Option[LocalDate] = None,
    notes: Option[String] = None,
    location: Option[String] = None,
    contactInfo: List[ContactEntry] = List.empty,
    isSelf: Boolean = false
  ): Person = Person(
    id = Id.generate[Person],
    name = name,
    nickname = nickname,
    howWeMet = howWeMet,
    birthday = birthday,
    notes = notes,
    location = location,
    contactInfo = contactInfo,
    isSelf = isSelf,
    archived = false
  )

  /**
   * Creates the "self" person - the network owner's representation in their own network.
   */
  def createSelf(
    name: String,
    nickname: Option[String] = None,
    birthday: Option[LocalDate] = None,
    notes: Option[String] = None,
    contactInfo: List[ContactEntry] = List.empty
  ): Person = create(
    name = name,
    nickname = nickname,
    birthday = birthday,
    notes = notes,
    contactInfo = contactInfo,
    isSelf = true
  )
}

// ============================================================================
// RELATIONSHIP
// ============================================================================

/**
 * Represents the nature of your connection to another person.
 * Labels describe what someone is to you: friend, family, coworker, etc.
 *
 * @param id       Unique identifier
 * @param name     The label text (e.g., "friend", "coworker", "mentor")
 * @param archived True if this label is archived (hidden from selection lists)
 */
case class RelationshipLabel(
  id: Id[RelationshipLabel],
  name: String,
  archived: Boolean
)

object RelationshipLabel {
  def create(name: String): RelationshipLabel = RelationshipLabel(
    id = Id.generate[RelationshipLabel],
    name = name,
    archived = false
  )

  /**
   * Common default labels to start with.
   * You can add more at any time.
   */
  val defaults: Set[RelationshipLabel] = Set(
    "me",
    "friend",
    "family",
    "coworker",
    "acquaintance",
    "mentor",
    "mentee",
    "neighbor",
    "former coworker",
    "romantic partner",
    "former romantic partner"
  ).map(create)
}

/**
 * Your relationship to another person.
 * This is directional: it represents YOUR relationship TO them.
 *
 * @param personId            The person this relationship is about
 * @param labels              Set of label IDs describing the relationship
 * @param reminderDays        Optional: remind me if I haven't interacted in this many days
 * @param interactionHistory  List of interactions, most recent first
 */
case class Relationship(
  personId: Id[Person],
  labels: Set[Id[RelationshipLabel]],
  reminderDays: Option[Int],
  interactionHistory: List[Interaction]
)

object Relationship {
  /**
   * Creates a new relationship with no interactions yet.
   */
  def create(
    personId: Id[Person],
    labels: Set[Id[RelationshipLabel]] = Set.empty,
    reminderDays: Option[Int] = None
  ): Relationship = Relationship(
    personId = personId,
    labels = labels,
    reminderDays = reminderDays,
    interactionHistory = List.empty
  )
}

// ============================================================================
// CIRCLE
// ============================================================================

/**
 * An organizational grouping of people.
 * Unlike labels (which describe what someone IS), circles are how YOU organize them.
 * Examples: "Putnam area", "potential farm collaborators", "weekly check-ins"
 *
 * @param id          Unique identifier
 * @param name        Circle name
 * @param description Optional description of the circle's purpose
 * @param memberIds   Set of person IDs in this circle
 * @param archived    True if this circle is archived (hidden from main view)
 */
case class Circle(
  id: Id[Circle],
  name: String,
  description: Option[String],
  memberIds: Set[Id[Person]],
  archived: Boolean
)

object Circle {
  def create(
    name: String,
    description: Option[String] = None,
    memberIds: Set[Id[Person]] = Set.empty
  ): Circle = Circle(
    id = Id.generate[Circle],
    name = name,
    description = description,
    memberIds = memberIds,
    archived = false
  )
}

// ============================================================================
// TOP-LEVEL CONTAINER
// ============================================================================

/**
 * Your entire personal network.
 * This is the single source of truth for all your relationship data.
 *
 * @param ownerId             The user who owns this network
 * @param selfId              This user's representation as a Person within their own network.
 *                            This allows the owner to have the same fields as anyone else
 *                            (birthday, notes, contact info, etc.) and maintains consistency.
 * @param people              All people in the network (including self), keyed by ID
 * @param relationships       Your relationships to others, keyed by their person ID
 * @param circles             Organizational groupings, keyed by ID
 * @param relationshipLabels  Available labels for describing relationships, keyed by ID
 * @param customContactTypes  User-defined contact types (e.g., "Discord"), keyed by ID
 */
case class Network(
  ownerId: Id[User],
  selfId: Id[Person],
  people: Map[Id[Person], Person],
  relationships: Map[Id[Person], Relationship],
  circles: Map[Id[Circle], Circle],
  relationshipLabels: Map[Id[RelationshipLabel], RelationshipLabel],
  customContactTypes: Map[Id[CustomContactType], CustomContactType]
)

object Network {
  /**
   * Creates a new network for a user.
   * Initializes with default relationship labels and no custom contact types.
   *
   * @param owner The user who will own this network
   * @param self  The Person representing the owner in their network (must have isSelf = true)
   */
  def create(owner: User, self: Person): Network = {
    require(self.isSelf, "The self person must have isSelf = true")

    val defaultLabels = RelationshipLabel.defaults
      .map(label => label.id -> label)
      .toMap

    Network(
      ownerId = owner.id,
      selfId = self.id,
      people = Map(self.id -> self),
      relationships = Map.empty,
      circles = Map.empty,
      relationshipLabels = defaultLabels,
      customContactTypes = Map.empty
    )
  }
}
