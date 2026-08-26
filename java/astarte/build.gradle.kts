plugins {
    id("buildlogic.java-library-conventions")
}

tasks.jar {
    manifest {
        attributes("Automatic-Module-Name" to "org.astarte.device")
    }
}
