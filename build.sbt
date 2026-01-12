ThisBuild / scalaVersion := "3.7.4"
ThisBuild / organization := "com.petros"

lazy val root = (project in file("."))
  .settings(
    name := "relationships",
    Compile / mainClass := Some("network.CLI"),
    libraryDependencies ++= Seq(
      "com.lihaoyi" %% "upickle" % "3.1.3",
      "org.scalatest" %% "scalatest" % "3.2.17" % Test
    )
  )
