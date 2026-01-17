package network

import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers
import java.time.LocalDate

class NetworkOpsSpec extends AnyFunSuite with Matchers {

  def getRight[L, R](either: Either[L, R]): R = either match {
    case Right(value) => value
    case Left(err) => fail(s"Expected Right but got Left($err)")
  }

  def createTestNetwork(): Network = {
    val owner = User.create("Petros", "petros@example.com")
    val self = Person.createSelf("Petros")
    Network.create(owner, self)
  }

  test("createUser creates user with valid fields") {
    val result = NetworkOps.createUser("Petros", "petros@example.com")
    result.isRight shouldBe true
    val user = getRight(result)
    user.name shouldBe "Petros"
  }

  test("addPerson creates a person with valid name") {
    val network = createTestNetwork()
    val result = NetworkOps.addPerson(network, "Alice")
    result.isRight shouldBe true
    val (updated, alice) = getRight(result)
    alice.name shouldBe "Alice"
    updated.people should have size 2
  }

  test("archivePerson fails for self") {
    val network = createTestNetwork()
    val result = NetworkOps.archivePerson(network, network.selfId)
    result.isLeft shouldBe true
  }

  test("logInPersonInteraction creates interaction with same location for both") {
    val network = createTestNetwork()
    val (networkWithAlice, alice) = getRight(NetworkOps.addPerson(network, "Alice"))
    
    val result = NetworkOps.logInPersonInteraction(
      networkWithAlice, alice.id,
      location = "Coffee shop",
      topics = Set("farming")
    )
    
    result.isRight shouldBe true
    val updated = getRight(result)
    val interaction = updated.relationships(alice.id).interactionHistory.head
    interaction.medium shouldBe InteractionMedium.InPerson
    interaction.myLocation shouldBe "Coffee shop"
    interaction.theirLocation shouldBe Some("Coffee shop")
  }

  test("logRemoteInteraction allows different locations") {
    val network = createTestNetwork()
    val (networkWithAlice, alice) = getRight(NetworkOps.addPerson(network, "Alice"))
    
    val result = NetworkOps.logRemoteInteraction(
      networkWithAlice, alice.id,
      medium = InteractionMedium.VideoCall,
      myLocation = "Home",
      theirLocation = Some("Office"),
      topics = Set("project")
    )
    
    result.isRight shouldBe true
    val updated = getRight(result)
    val interaction = updated.relationships(alice.id).interactionHistory.head
    interaction.medium shouldBe InteractionMedium.VideoCall
    interaction.myLocation shouldBe "Home"
    interaction.theirLocation shouldBe Some("Office")
  }

  test("logRemoteInteraction fails for InPerson medium") {
    val network = createTestNetwork()
    val (networkWithAlice, alice) = getRight(NetworkOps.addPerson(network, "Alice"))
    
    val result = NetworkOps.logRemoteInteraction(
      networkWithAlice, alice.id,
      medium = InteractionMedium.InPerson,
      myLocation = "Home",
      theirLocation = None,
      topics = Set("topic")
    )
    
    result.isLeft shouldBe true
  }

  test("setLabels replaces all labels") {
    val network = createTestNetwork()
    val (networkWithAlice, alice) = getRight(NetworkOps.addPerson(network, "Alice"))
    val friendLabel = networkWithAlice.relationshipLabels.values.find(_.name == "friend").get
    val familyLabel = networkWithAlice.relationshipLabels.values.find(_.name == "family").get
    
    val n1 = getRight(NetworkOps.setLabels(networkWithAlice, alice.id, Set(friendLabel.id)))
    n1.relationships(alice.id).labels shouldBe Set(friendLabel.id)
    
    val n2 = getRight(NetworkOps.setLabels(n1, alice.id, Set(familyLabel.id)))
    n2.relationships(alice.id).labels shouldBe Set(familyLabel.id)
  }

  test("createCircle with initial members") {
    val network = createTestNetwork()
    val (n1, alice) = getRight(NetworkOps.addPerson(network, "Alice"))
    val (n2, bob) = getRight(NetworkOps.addPerson(n1, "Bob"))
    
    val result = NetworkOps.createCircle(n2, "Friends", memberIds = Set(alice.id, bob.id))
    
    result.isRight shouldBe true
    val (_, circle) = getRight(result)
    circle.memberIds shouldBe Set(alice.id, bob.id)
  }

  test("archiveCircle sets archived flag") {
    val network = createTestNetwork()
    val (networkWithCircle, circle) = getRight(NetworkOps.createCircle(network, "Test"))
    
    val result = NetworkOps.archiveCircle(networkWithCircle, circle.id)
    
    result.isRight shouldBe true
    val updated = getRight(result)
    updated.circles(circle.id).archived shouldBe true
  }

  test("unarchiveCircle clears archived flag") {
    val network = createTestNetwork()
    val (n1, circle) = getRight(NetworkOps.createCircle(network, "Test"))
    val n2 = getRight(NetworkOps.archiveCircle(n1, circle.id))
    
    val result = NetworkOps.unarchiveCircle(n2, circle.id)
    
    result.isRight shouldBe true
    val updated = getRight(result)
    updated.circles(circle.id).archived shouldBe false
  }

  test("setCircleMembers replaces all members") {
    val network = createTestNetwork()
    val (n1, alice) = getRight(NetworkOps.addPerson(network, "Alice"))
    val (n2, bob) = getRight(NetworkOps.addPerson(n1, "Bob"))
    val (n3, circle) = getRight(NetworkOps.createCircle(n2, "Test", memberIds = Set(alice.id)))
    
    val result = NetworkOps.setCircleMembers(n3, circle.id, Set(bob.id))
    
    result.isRight shouldBe true
    val updated = getRight(result)
    updated.circles(circle.id).memberIds shouldBe Set(bob.id)
  }

  test("addLabel creates new label") {
    val network = createTestNetwork()
    val initialCount = network.relationshipLabels.size
    
    val result = NetworkOps.addLabel(network, "farming partner")
    
    result.isRight shouldBe true
    val (updated, label) = getRight(result)
    label.name shouldBe "farming partner"
    label.archived shouldBe false
    updated.relationshipLabels should have size (initialCount + 1)
  }

  test("updateLabel changes label name") {
    val network = createTestNetwork()
    val friendLabel = network.relationshipLabels.values.find(_.name == "friend").get
    
    val result = NetworkOps.updateLabel(network, friendLabel.id, name = Some("close friend"))
    
    result.isRight shouldBe true
    val updated = getRight(result)
    updated.relationshipLabels(friendLabel.id).name shouldBe "close friend"
  }

  test("archiveLabel sets archived to true") {
    val network = createTestNetwork()
    val friendLabel = network.relationshipLabels.values.find(_.name == "friend").get
    
    val result = NetworkOps.archiveLabel(network, friendLabel.id)
    
    result.isRight shouldBe true
    val updated = getRight(result)
    updated.relationshipLabels(friendLabel.id).archived shouldBe true
  }

  test("unarchiveLabel sets archived to false") {
    val network = createTestNetwork()
    val friendLabel = network.relationshipLabels.values.find(_.name == "friend").get
    
    // First archive
    val archived = getRight(NetworkOps.archiveLabel(network, friendLabel.id))
    archived.relationshipLabels(friendLabel.id).archived shouldBe true
    
    // Then unarchive
    val result = NetworkOps.unarchiveLabel(archived, friendLabel.id)
    result.isRight shouldBe true
    val updated = getRight(result)
    updated.relationshipLabels(friendLabel.id).archived shouldBe false
  }

  test("setRelationship works for self") {
    val network = createTestNetwork()
    val friendLabel = network.relationshipLabels.values.find(_.name == "friend").get
    
    val result = NetworkOps.setRelationship(network, network.selfId, Set(friendLabel.id), Some(7))
    
    result.isRight shouldBe true
    val updated = getRight(result)
    updated.relationships(network.selfId).labels shouldBe Set(friendLabel.id)
    updated.relationships(network.selfId).reminderDays shouldBe Some(7)
  }

  test("logInPersonInteraction works for self") {
    val network = createTestNetwork()
    
    val result = NetworkOps.logInPersonInteraction(
      network, network.selfId,
      location = "Home",
      topics = Set("journaling", "reflection")
    )
    
    result.isRight shouldBe true
    val updated = getRight(result)
    val interaction = updated.relationships(network.selfId).interactionHistory.head
    interaction.topics should contain allOf ("journaling", "reflection")
  }
}
