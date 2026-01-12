package network

import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers
import java.time.LocalDate

class JsonCodecsSpec extends AnyFunSuite with Matchers {

  import JsonCodecs.given
  import upickle.default.*

  def getRight[L, R](either: Either[L, R]): R = either match {
    case Right(value) => value
    case Left(err) => fail(s"Expected Right but got Left($err)")
  }

  test("LocalDate roundtrips correctly") {
    val date = LocalDate.of(2025, 1, 15)
    val json = write(date)
    val restored = read[LocalDate](json)
    restored shouldBe date
  }

  test("InteractionMedium roundtrips correctly") {
    for (medium <- InteractionMedium.all) {
      val json = write(medium)
      val restored = read[InteractionMedium](json)
      restored shouldBe medium
    }
  }

  test("Interaction with medium and locations roundtrips correctly") {
    val interaction = Interaction.createRemote(
      medium = InteractionMedium.VideoCall,
      myLocation = "Home",
      theirLocation = Some("Office"),
      topics = Set("project", "planning"),
      note = Some("Good call")
    )
    
    val json = write(interaction)
    val restored = read[Interaction](json)
    
    restored shouldBe interaction
    restored.medium shouldBe InteractionMedium.VideoCall
    restored.myLocation shouldBe "Home"
    restored.theirLocation shouldBe Some("Office")
  }

  test("Interaction in-person roundtrips correctly") {
    val interaction = Interaction.createInPerson(
      location = "Coffee shop",
      topics = Set("life")
    )
    
    val json = write(interaction)
    val restored = read[Interaction](json)
    
    restored.medium shouldBe InteractionMedium.InPerson
    restored.myLocation shouldBe "Coffee shop"
    restored.theirLocation shouldBe Some("Coffee shop")
  }

  test("Circle with archived flag roundtrips correctly") {
    val circle = Circle.create("Test Circle", memberIds = Set(Id.generate[Person]))
    val archived = circle.copy(archived = true)
    
    val json = write(archived)
    val restored = read[Circle](json)
    
    restored.archived shouldBe true
    restored.name shouldBe "Test Circle"
  }

  test("Populated network with new fields roundtrips correctly") {
    val owner = User.create("Petros", "petros@example.com")
    val self = Person.createSelf("Petros")
    var network = Network.create(owner, self)
    
    val (n1, alice) = getRight(NetworkOps.addPerson(network, "Alice"))
    network = n1
    
    // Log interaction with new medium
    val n2 = getRight(NetworkOps.logRemoteInteraction(
      network, alice.id,
      medium = InteractionMedium.Text,
      myLocation = "Home",
      theirLocation = Some("Work"),
      topics = Set("hello")
    ))
    network = n2
    
    // Create and archive a circle
    val (n3, circle) = getRight(NetworkOps.createCircle(network, "Friends", memberIds = Set(alice.id)))
    val n4 = getRight(NetworkOps.archiveCircle(n3, circle.id))
    network = n4
    
    // Roundtrip
    val json = write(network)
    val restored = read[Network](json)
    
    restored.people.size shouldBe network.people.size
    restored.circles(circle.id).archived shouldBe true
    
    val interaction = restored.relationships(alice.id).interactionHistory.head
    interaction.medium shouldBe InteractionMedium.Text
    interaction.myLocation shouldBe "Home"
    interaction.theirLocation shouldBe Some("Work")
  }

  test("toJson and fromJson work correctly") {
    val owner = User.create("Petros", "petros@example.com")
    val self = Person.createSelf("Petros")
    val network = Network.create(owner, self)
    
    val json = JsonCodecs.toJson(network)
    val result = JsonCodecs.fromJson(json)
    
    result.isRight shouldBe true
    getRight(result).selfId shouldBe network.selfId
  }
}
