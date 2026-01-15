val scala3Version = "3.7.4"

lazy val root = project
  .in(file("."))
  .settings(
    name := "prm",
    version := "0.1.0",
    scalaVersion := scala3Version,
    
    libraryDependencies ++= Seq(
      "com.lihaoyi" %% "upickle" % "4.1.0",
      "org.scalatest" %% "scalatest" % "3.2.19" % Test
    ),
    
    // Main class for running
    Compile / mainClass := Some("network.CLI"),
    
    // Assembly settings (for fat JAR)
    assembly / assemblyJarName := "prm.jar",
    assembly / mainClass := Some("network.CLI"),
    assembly / assemblyMergeStrategy := {
      case PathList("META-INF", xs @ _*) => MergeStrategy.discard
      case x => MergeStrategy.first
    }
  )
