#include <ESP8266WiFi.h>

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
  Serial.println("DATA:TEMP=25.4");
  delay(2000);
}