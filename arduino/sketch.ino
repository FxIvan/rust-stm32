#include <ESP8266WiFi.h>
#include <ESP8266HTTPClient.h>
#include <ArduinoJson.h>

const char* ssid = "IvanAlmendra";
const char* pass = "ivan2022";

void setup() {
  Serial.begin(115200);
  WiFi.begin(ssid, pass);
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
  }
}

void fetchAndSend() {
  WiFiClient client;  // ← HTTP simple, sin SSL
  HTTPClient http;
  http.begin(client, "http://storage.googleapis.com/demo-pipeline-datos-proyecto-laboratorio-467421/status-services_1.json");
  http.setTimeout(10000);

  int httpCode = http.GET();

  if (httpCode == HTTP_CODE_OK) {
    String payload = http.getString();
    StaticJsonDocument<512> doc;
    DeserializationError error = deserializeJson(doc, payload);

    if (!error) {
      const char* login     =     doc["microservice-login"];
      // const char* notif     = doc["microservice-notification"] | "NULL";
      // const char* pay       = doc["microservice-payment"]      | "NULL";
      // const char* analytics = doc["microservice-analytics"]    | "NULL";

      Serial.print("LOGIN=");     Serial.println(login);
      // Serial.print("NOTIF=");     Serial.println(notif);
      // Serial.print("PAY=");       Serial.println(pay);
      // Serial.print("ANALYTICS="); Serial.println(analytics);

    } else {
      Serial.println("JSON_ERROR");
    }
  } else {
    Serial.print("HTTP_ERROR=");
    Serial.println(httpCode);
  }

  http.end();
}

void loop() {
  if (WiFi.status() == WL_CONNECTED) {
    fetchAndSend();
  } else {
    WiFi.reconnect();
    delay(5000);
  }
  delay(10000);
}