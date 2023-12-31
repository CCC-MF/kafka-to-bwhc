# Apache Kafka to bwHC-Backend Gateway

Diese Anwendung wartet auf neue Inhalte in einem Apache Kafka Topic und entnimmt enthaltene MTB-Files und leitet diese
an das bwHC-Backend weiter.

Der Key wird beibehalten und die Rückantwort vom bwHC-Backend wird in ein konfiguriertes Response-Topic versendet,
sodass der ETL-Processor darauf reagieren kann.

Verwendung im Zusammenspiel mit https://github.com/CCC-MF/etl-processor

### Einordnung innerhalb einer DNPM-ETL-Strecke

Diese Anwendung erlaubt das Entgegennehmen von Kafka-Records mit MTB-File-Inhalten und Weiterleiten dieser Inhalte mit einem
HTTP/REST-Request an die bwHC-Software.
Antworten von der bwHC-Software werden wiederum als Kafka-Record zurückgesendet.

Sie dient hierbei als Baustein und zum Übersetzen von Nachrichten innerhalb einer gedachten DNPM-ETL-Strecke.

![Modell DNPM-ETL-Strecke](docs/etl.png)

## Konfiguration

Die Anwendung lässt sich mit Umgebungsvariablen konfigurieren.

* `APP_REST_URI`: URI der zu benutzenden API der bwHC-Backend-Instanz. z.B.: `http://localhost:9000/bwhc/etl/api`
* `APP_KAFKA_TOPIC`: Zu verwendendes Topic zum Warten auf neue Anfragen
* `APP_KAFKA_RESPONSE_TOPIC`: Topic zum Versenden der Antworten. Standardwert: `APP_KAFKA_TOPIC` mit Anhang "_response".
* `APP_KAFKA_GROUP_ID`: Kafka GroupID des Consumers. Standardwert: `APP_KAFKA_TOPIC` mit Anhang "_group".
* `APP_KAFKA_SERVERS`: Zu verwendende Kafka-Bootstrap-Server als kommagetrennte Liste

## Besonderheiten

Konnte keine HTTP-Verbindung zum bwHC-Backend aufgebaut werden, wird eine Fehlermeldung mit Status-Code `900` zurück gesendet.

Hierdurch ist es dem ETL-Prozessor möglich, diesen Fehler zu identifizieren und entsprechend zu loggen.