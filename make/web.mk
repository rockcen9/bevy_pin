build:
	bevy build --release --yes --features web,backend web --bundle

build-release:
	bevy build --release --yes --features web,backend web --bundle

run:
	bevy run --features web,backend web

release:
	bevy run --release --yes --features web,backend web --bundle