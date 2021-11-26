#include <Arduino.h>
#include <ESP32Lib.h>

const int controllerPin = 21;
const int clockPin = 4;
const int latchPin = 15; 

GameControllers controllers;
/* SNES wires pinout
 * THING    INTERNAL HEADERS
[ ] VCC		WHITE    RED
[ ] CLOCK	BLUE     BLUE
[ ] LATCH	YELLOW   YELLOW
[ ] DATA	RED      GREEN
---
[ ] n/a
[ ] n/a
[ ] GND		BROWN    ORANGE
*/

void setup()
{
	controllers.init(latchPin, clockPin);
	controllers.setController(0, GameControllers::SNES, controllerPin);
	Serial.begin(115200);
}
void loop() {
	controllers.poll();
	Serial.write("CKIE");
	for (int i = 0; i < 12; i++) {
		Serial.write(controllers.buttons[0][i]);
	}

	delay(10);
}
