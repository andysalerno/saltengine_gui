#!/usr/bin/bash

cargo build \
	&& cp target/debug/agent.dll ../DummyProj/networking \
	&& cp target/debug/card_instance.dll ../DummyProj/card

