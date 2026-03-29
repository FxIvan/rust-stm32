#include <ESP8266WiFi.h>
#include <ESP8266HTTPClient.h>

const char* ssid = "IvanAlmendra";
const char* pass = "ivan2022";

void setup() {
  Serial.begin(115200);

  WiFi.begin(ssid, pass);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
  }

  Serial.println("WIFI_OK");
}

void loop() {
  if (WiFi.status() == WL_CONNECTED) {

    WiFiClient client;
    HTTPClient http;

    http.begin(client, "http://api.ipify.org");
    int httpCode = http.GET();

    if (httpCode > 0) {
      String ip = http.getString();

      // Mandar al STM32
Serial.print("IP:");
Serial.println(ip);
    } else {
      Serial.println("ERROR_HTTP");
    }

    http.end();
  } else {
    Serial.println("WIFI_LOST");
  }

  delay(10000); // cada 10 segundos
}