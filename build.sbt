val scala3Version = "3.7.0"

lazy val root = project
  .in(file("."))
  .enablePlugins(NativeImagePlugin)
  .settings(
    name := "relationships",
    version := "0.1.0",
    scalaVersion := scala3Version,
    
    libraryDependencies ++= Seq(
      "com.lihaoyi" %% "upickle" % "4.1.0",
      "org.scalatest" %% "scalatest" % "3.2.19" % Test
    ),
    
    // Main class for running
    Compile / mainClass := Some("network.CLI"),
    
    // Native image settings
    nativeImageOptions ++= Seq(
      "--no-fallback",
      "--initialize-at-build-time",
      "-H:+ReportExceptionStackTraces"
    ),
    // Use graalvm-community edition which is available via Coursier
    nativeImageJvm := "graalvm-community",
    nativeImageVersion := "21.0.2"
  )
