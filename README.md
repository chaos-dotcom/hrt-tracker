# hrt tracker
# (planned) features
* localstorage, or some simple shared one you can copy / save across browsers. no login that's not the point
* set reminders for taking hrt? but not really ideal as it's a website
* simplest part:
	* enter → get asked to put in current dosage, past blood results (or import)
	* ask if you wanna set up a tracker → track various configurable things
	* if not, simple mode -
		* you just input whether injected / oral
			* → time of ingestion / injection and type of hormone. ev oral? injected? hemihydrate?
* change in graphs overtime for trackable stuff

## trackable stuff
1. weight
2. height
3. (bmi → calculated)
4. underbust
5. bust
6. → bra size
7. bideltoid
8. waist
9. hip


## routes
index - onboarding if not setup, if setup then dashboard view, buttons to add blood test or check off dosage

create/blood-test → create a new blood test entry. feature → read from pdf / image of an actual medical result using ocr. free vision model? or local? or tesseract?
create/dosage → create a new daily hrt dosage

tracker/ → view blood tests, view dosages

/settings → idk theme option maybe? defaulting to rose pine for myself

optional:
* /backup → import, export localstorage to a json
* use remotestorage with a small self hosted server
