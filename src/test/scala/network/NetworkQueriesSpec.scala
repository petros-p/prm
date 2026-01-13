package network

import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers
import java.time.LocalDate

class NetworkQueriesSpec extends AnyFunSuite with Matchers {

  def getRight[L, R](either: Either[L, R]): R = either match {
    case Right(value) => value
    case Left(err) => fail(s"Expected Right but got Left($err)")
  }

  def createPopulatedNetwork(): Network = {
    val owner = User.create("Petros", "petros@example.com")
    val self = Person.createSelf("Petros")
    var network = Network.create(owner, self)
    
    val (n1, alice) = getRight(NetworkOps.addPerson(network, "Alice", defaultLocation = Some("Coffee shop")))
    val (n2, bob) = getRight(NetworkOps.addPerson(n1, "Bob"))
    val (n3, carol) = getRight(NetworkOps.addPerson(n2, "Carol", nickname = Some("C")))
    network = n3
    
    val friendLabel = network.relationshipLabels.values.find(_.name == "friend").get
    val familyLabel = network.relationshipLabels.values.find(_.name == "family").get
    
    val n4 = getRight(NetworkOps.setRelationship(network, alice.id, Set(friendLabel.id), Some(7)))
    val n5 = getRight(NetworkOps.setRelationship(n4, bob.id, Set(friendLabel.id, familyLabel.id), Some(30)))
    val n6 = getRight(NetworkOps.setRelationship(n5, carol.id, Set(familyLabel.id)))
    network = n6
    
    val n7 = getRight(NetworkOps.logInPersonInteraction(network, alice.id, "Park", Set("walking"), date = LocalDate.now().minusDays(3)))
    val n8 = getRight(NetworkOps.logInPersonInteraction(n7, bob.id, "Home", Set("dinner"), date = LocalDate.now().minusDays(45)))
    network = n8
    
    network
  }

  test("activePeople returns non-archived people including self") {
    val network = createPopulatedNetwork()
    val active = NetworkQueries.activePeople(network)
    active.map(_.name) should contain allOf ("Alice", "Bob", "Carol", "Petros")
  }

  test("activeCircles returns non-archived circles") {
    val network = createPopulatedNetwork()
    val (n1, circle1) = getRight(NetworkOps.createCircle(network, "Active Circle"))
    val (n2, circle2) = getRight(NetworkOps.createCircle(n1, "Archived Circle"))
    val n3 = getRight(NetworkOps.archiveCircle(n2, circle2.id))
    
    val active = NetworkQueries.activeCircles(n3)
    active.map(_.name) should contain ("Active Circle")
    active.map(_.name) should not contain "Archived Circle"
  }

  test("archivedCircles returns only archived circles") {
    val network = createPopulatedNetwork()
    val (n1, circle1) = getRight(NetworkOps.createCircle(network, "Active Circle"))
    val (n2, circle2) = getRight(NetworkOps.createCircle(n1, "Archived Circle"))
    val n3 = getRight(NetworkOps.archiveCircle(n2, circle2.id))
    
    val archived = NetworkQueries.archivedCircles(n3)
    archived.map(_.name) should contain ("Archived Circle")
    archived.map(_.name) should not contain "Active Circle"
  }

  test("reminderStatus returns NeverContacted when person has no interactions") {
    val network = createPopulatedNetwork()
    val carol = network.people.values.find(_.name == "Carol").get
    val updated = getRight(NetworkOps.setReminder(network, carol.id, Some(7)))
    
    val status = NetworkQueries.reminderStatus(updated, carol.id)
    
    status.isDefined shouldBe true
    status.get.daysSinceLastInteraction shouldBe None
    status.get.overdueStatus shouldBe OverdueStatus.NeverContacted
  }

  test("peopleNeedingReminder includes NeverContacted people") {
    val network = createPopulatedNetwork()
    val carol = network.people.values.find(_.name == "Carol").get
    val updated = getRight(NetworkOps.setReminder(network, carol.id, Some(7)))
    
    val needing = NetworkQueries.peopleNeedingReminder(updated)
    needing.map(_.person.name) should contain ("Carol")
    needing.head.overdueStatus shouldBe OverdueStatus.NeverContacted
  }

  test("stats returns correct counts including circle archiving") {
    val network = createPopulatedNetwork()
    val (n1, _) = getRight(NetworkOps.createCircle(network, "Active"))
    val (n2, circle2) = getRight(NetworkOps.createCircle(n1, "ToArchive"))
    val n3 = getRight(NetworkOps.archiveCircle(n2, circle2.id))
    
    val s = NetworkQueries.stats(n3)
    
    s.totalPeople shouldBe 4  // Alice, Bob, Carol, Petros (self)
    s.activePeople shouldBe 4
    s.totalCircles shouldBe 2
    s.activeCircles shouldBe 1
    s.archivedCircles shouldBe 1
  }
}
