package network

import java.time.LocalDate
import java.time.temporal.ChronoUnit

// ============================================================================
// QUERY FUNCTIONS
// ============================================================================

/**
 * Represents whether someone is overdue for contact and by how much.
 */
sealed trait OverdueStatus

object OverdueStatus {
  /** Person has been contacted before; days is positive if overdue, negative if not yet due */
  case class DaysOverdue(days: Long) extends OverdueStatus
  
  /** Person has never been contacted, so they are inherently overdue */
  case object NeverContacted extends OverdueStatus
}

/**
 * Information about a relationship's reminder status.
 */
case class ReminderStatus(
  person: Person,
  relationship: Relationship,
  reminderDays: Int,
  daysSinceLastInteraction: Option[Long],  // None if no interactions
  overdueStatus: OverdueStatus
)

/**
 * Pure query functions for exploring and filtering the network.
 * These don't modify anything - they just return views of the data.
 */
object NetworkQueries {

  // --------------------------------------------------------------------------
  // PERSON QUERIES
  // --------------------------------------------------------------------------

  /**
   * Gets all active (non-archived) people, including self.
   */
  def activePeople(network: Network): List[Person] =
    network.people.values
      .filter(p => !p.archived)
      .toList
      .sortBy(_.name)

  /**
   * Gets all archived people.
   */
  def archivedPeople(network: Network): List[Person] =
    network.people.values
      .filter(_.archived)
      .toList
      .sortBy(_.name)

  /**
   * Gets the self person.
   */
  def self(network: Network): Person =
    network.people(network.selfId)

  /**
   * Finds a person by name (case-insensitive, partial match).
   */
  def findByName(network: Network, query: String): List[Person] = {
    val lowerQuery = query.toLowerCase.trim
    network.people.values
      .filter { p =>
        p.name.toLowerCase.contains(lowerQuery) ||
        p.nickname.exists(_.toLowerCase.contains(lowerQuery))
      }
      .toList
      .sortBy(_.name)
  }

  /**
   * Gets a person by their ID.
   */
  def getPerson(network: Network, personId: Id[Person]): Option[Person] =
    network.people.get(personId)

  // --------------------------------------------------------------------------
  // CONTACT INFO QUERIES
  // --------------------------------------------------------------------------

  /**
   * Gets all phone numbers for a person.
   */
  def phonesFor(network: Network, personId: Id[Person]): List[ContactEntry] =
    network.people.get(personId)
      .map(_.contactInfo.filter(_.contactType == ContactType.Phone))
      .getOrElse(List.empty)

  /**
   * Gets all email addresses for a person.
   */
  def emailsFor(network: Network, personId: Id[Person]): List[ContactEntry] =
    network.people.get(personId)
      .map(_.contactInfo.filter(_.contactType == ContactType.Email))
      .getOrElse(List.empty)

  /**
   * Gets all physical addresses for a person.
   */
  def addressesFor(network: Network, personId: Id[Person]): List[ContactEntry] =
    network.people.get(personId)
      .map(_.contactInfo.filter(_.contactType == ContactType.PhysicalAddress))
      .getOrElse(List.empty)

  /**
   * Gets all custom contact entries of a specific type for a person.
   */
  def customContactsFor(
    network: Network,
    personId: Id[Person],
    typeId: Id[CustomContactType]
  ): List[ContactEntry] =
    network.people.get(personId)
      .map(_.contactInfo.filter(_.contactType == ContactType.Custom(typeId)))
      .getOrElse(List.empty)

  /**
   * Gets all contact entries for a person.
   */
  def allContactsFor(network: Network, personId: Id[Person]): List[ContactEntry] =
    network.people.get(personId)
      .map(_.contactInfo)
      .getOrElse(List.empty)

  /**
   * Finds people who have a specific custom contact type (e.g., everyone with Discord).
   */
  def peopleWithCustomContactType(
    network: Network,
    typeId: Id[CustomContactType]
  ): List[Person] =
    network.people.values
      .filter { p =>
        !p.archived && p.contactInfo.exists(_.contactType == ContactType.Custom(typeId))
      }
      .toList
      .sortBy(_.name)

  /**
   * Gets the name of a custom contact type.
   */
  def customContactTypeName(network: Network, typeId: Id[CustomContactType]): Option[String] =
    network.customContactTypes.get(typeId).map(_.name)

  // --------------------------------------------------------------------------
  // RELATIONSHIP QUERIES
  // --------------------------------------------------------------------------

  /**
   * Gets the relationship with a person, if one exists.
   */
  def getRelationship(network: Network, personId: Id[Person]): Option[Relationship] =
    network.relationships.get(personId)

  /**
   * Gets all people with a specific label.
   */
  def peopleWithLabel(network: Network, labelId: Id[RelationshipLabel]): List[Person] =
    network.relationships
      .filter { case (_, rel) => rel.labels.contains(labelId) }
      .flatMap { case (personId, _) => network.people.get(personId) }
      .filter(!_.archived)
      .toList
      .sortBy(_.name)

  /**
   * Gets all people with a specific label name (convenience method).
   */
  def peopleWithLabelName(network: Network, labelName: String): List[Person] = {
    val lowerName = labelName.toLowerCase.trim
    network.relationshipLabels.values
      .find(_.name.toLowerCase == lowerName)
      .map(label => peopleWithLabel(network, label.id))
      .getOrElse(List.empty)
  }

  /**
   * Gets the labels for a relationship as actual RelationshipLabel objects.
   */
  def labelsFor(network: Network, personId: Id[Person]): Set[RelationshipLabel] =
    network.relationships.get(personId)
      .map(_.labels.flatMap(network.relationshipLabels.get))
      .getOrElse(Set.empty)

  // --------------------------------------------------------------------------
  // CIRCLE QUERIES
  // --------------------------------------------------------------------------

  /**
   * Gets all active (non-archived) circles.
   */
  def activeCircles(network: Network): List[Circle] =
    network.circles.values
      .filter(!_.archived)
      .toList
      .sortBy(_.name)

  /**
   * Gets all archived circles.
   */
  def archivedCircles(network: Network): List[Circle] =
    network.circles.values
      .filter(_.archived)
      .toList
      .sortBy(_.name)

  /**
   * Gets all members of a circle.
   */
  def circleMembers(network: Network, circleId: Id[Circle]): List[Person] =
    network.circles.get(circleId)
      .map(_.memberIds.flatMap(network.people.get).toList.sortBy(_.name))
      .getOrElse(List.empty)

  /**
   * Gets all circles a person belongs to (active circles only).
   */
  def circlesFor(network: Network, personId: Id[Person]): List[Circle] =
    network.circles.values
      .filter(c => !c.archived && c.memberIds.contains(personId))
      .toList
      .sortBy(_.name)

  /**
   * Finds a circle by name (case-insensitive, exact match).
   * Searches both active and archived circles.
   */
  def findCircleByName(network: Network, name: String): Option[Circle] = {
    val lowerName = name.toLowerCase.trim
    network.circles.values.find(_.name.toLowerCase == lowerName)
  }

  /**
   * Finds an active circle by name (case-insensitive, partial match).
   */
  def findActiveCircleByName(network: Network, query: String): List[Circle] = {
    val lowerQuery = query.toLowerCase.trim
    network.circles.values
      .filter(c => !c.archived && c.name.toLowerCase.contains(lowerQuery))
      .toList
      .sortBy(_.name)
  }

  // --------------------------------------------------------------------------
  // INTERACTION QUERIES
  // --------------------------------------------------------------------------

  /**
   * Gets all interactions with a person, most recent first.
   */
  def interactionsWith(network: Network, personId: Id[Person]): List[Interaction] =
    network.relationships.get(personId)
      .map(_.interactionHistory)
      .getOrElse(List.empty)

  /**
   * Gets the most recent interaction with a person, if any.
   */
  def lastInteractionWith(network: Network, personId: Id[Person]): Option[Interaction] =
    interactionsWith(network, personId).headOption

  /**
   * Gets the date of the last interaction with a person, if any.
   */
  def lastInteractionDate(network: Network, personId: Id[Person]): Option[LocalDate] =
    lastInteractionWith(network, personId).map(_.date)

  /**
   * Calculates days since the last interaction with a person.
   * Returns None if there have been no interactions.
   */
  def daysSinceInteraction(
    network: Network,
    personId: Id[Person],
    asOf: LocalDate = LocalDate.now()
  ): Option[Long] =
    lastInteractionDate(network, personId).map(date => ChronoUnit.DAYS.between(date, asOf))

  /**
   * Gets all interactions across all relationships within a date range.
   */
  def interactionsInRange(
    network: Network,
    from: LocalDate,
    to: LocalDate
  ): List[(Person, Interaction)] =
    network.relationships.toList.flatMap { case (personId, rel) =>
      network.people.get(personId).toList.flatMap { person =>
        rel.interactionHistory
          .filter(i => !i.date.isBefore(from) && !i.date.isAfter(to))
          .map(i => (person, i))
      }
    }.sortBy { case (_, i) => i.date }(using Ordering[LocalDate].reverse)

  // --------------------------------------------------------------------------
  // REMINDER QUERIES
  // --------------------------------------------------------------------------

  /**
   * Gets reminder status for a specific person.
   */
  def reminderStatus(
    network: Network,
    personId: Id[Person],
    asOf: LocalDate = LocalDate.now()
  ): Option[ReminderStatus] =
    for {
      person <- network.people.get(personId)
      rel <- network.relationships.get(personId)
      days <- rel.reminderDays
    } yield {
      val daysSince = daysSinceInteraction(network, personId, asOf)
      val overdueStatus = daysSince match {
        case Some(d) => OverdueStatus.DaysOverdue(d - days)
        case None => OverdueStatus.NeverContacted
      }
      ReminderStatus(person, rel, days, daysSince, overdueStatus)
    }

  /**
   * Gets all people who need a reminder (overdue for interaction).
   * Includes people with no interactions if they have a reminder set.
   * Sorted by most overdue first (NeverContacted at top, then by days overdue).
   */
  def peopleNeedingReminder(
    network: Network,
    asOf: LocalDate = LocalDate.now()
  ): List[ReminderStatus] =
    network.relationships.toList
      .flatMap { case (personId, _) => reminderStatus(network, personId, asOf) }
      .filter { status =>
        status.overdueStatus match {
          case OverdueStatus.NeverContacted => true
          case OverdueStatus.DaysOverdue(days) => days > 0
        }
      }
      .filter(!_.person.archived)
      .sortBy { status =>
        status.overdueStatus match {
          case OverdueStatus.NeverContacted => Long.MinValue // Sort to top
          case OverdueStatus.DaysOverdue(days) => -days      // Most overdue first
        }
      }

  /**
   * Gets all people with reminders, regardless of whether they're due.
   * Sorted by most overdue first (NeverContacted at top, then by days overdue).
   */
  def allReminders(
    network: Network,
    asOf: LocalDate = LocalDate.now()
  ): List[ReminderStatus] =
    network.relationships.toList
      .flatMap { case (personId, _) => reminderStatus(network, personId, asOf) }
      .filter(!_.person.archived)
      .sortBy { status =>
        status.overdueStatus match {
          case OverdueStatus.NeverContacted => Long.MinValue
          case OverdueStatus.DaysOverdue(days) => -days
        }
      }

  /**
   * Gets people you haven't interacted with in a given number of days.
   * Unlike reminders, this checks ALL relationships, not just those with reminders set.
   */
  def notContactedIn(
    network: Network,
    days: Long,
    asOf: LocalDate = LocalDate.now()
  ): List[(Person, Option[Long])] =
    network.relationships.toList
      .flatMap { case (personId, _) =>
        network.people.get(personId).map { person =>
          val daysSince = daysSinceInteraction(network, personId, asOf)
          (person, daysSince)
        }
      }
      .filter { case (person, daysSince) =>
        !person.archived && daysSince.forall(_ >= days)
      }
      .sortBy { case (_, days) => -days.getOrElse(Long.MaxValue) }

  // --------------------------------------------------------------------------
  // STATISTICS
  // --------------------------------------------------------------------------

  /**
   * Basic statistics about the network.
   */
  case class NetworkStats(
    totalPeople: Int,
    activePeople: Int,
    archivedPeople: Int,
    totalRelationships: Int,
    totalInteractions: Int,
    totalCircles: Int,
    activeCircles: Int,
    archivedCircles: Int,
    remindersOverdue: Int,
    customContactTypes: Int
  )

  def stats(network: Network): NetworkStats = {
    val people = network.people.values
    val circles = network.circles.values
    NetworkStats(
      totalPeople = people.size,
      activePeople = people.count(!_.archived),
      archivedPeople = people.count(_.archived),
      totalRelationships = network.relationships.size,
      totalInteractions = network.relationships.values.map(_.interactionHistory.size).sum,
      totalCircles = circles.size,
      activeCircles = circles.count(!_.archived),
      archivedCircles = circles.count(_.archived),
      remindersOverdue = peopleNeedingReminder(network).size,
      customContactTypes = network.customContactTypes.size
    )
  }
}
