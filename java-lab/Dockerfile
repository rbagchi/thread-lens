FROM openjdk:17-jdk-slim

WORKDIR /app

COPY gradlew ./
COPY gradle/ ./gradle/
COPY build.gradle ./
COPY settings.gradle ./

RUN ./gradlew bootJar

CMD ["java", "-jar", "build/libs/*.jar"]
