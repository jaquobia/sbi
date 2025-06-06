{
	pkgs,
	wayland,
	libxkbcommon,
	vulkan-headers,
	vulkan-loader,
	libGL,
	makeWrapper,
	patchelf,
	pkg-config
}:
let
	dlopenLibraries = [
		wayland
		libxkbcommon
		vulkan-headers
		vulkan-loader
		libGL
	];
in
pkgs.rustPlatform.buildRustPackage {
	pname = "sbi";
	version = "0.0.0";
	cargoLock.lockFile = ./Cargo.lock;
	src = pkgs.lib.cleanSource ./.;

	nativeBuildInputs = [
		patchelf
		makeWrapper
		pkg-config
	];

	# buildInputs = [
	# ] ++ dlopenLibraries;

	# patchelf --shrink-rpath in phaseFixup just removes the paths as for some reason these are not declared as necessary.
	# RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dlopenLibraries}";

	postFixup = ''
		patchelf $out/bin/.sbi-wrapped --add-rpath ${pkgs.lib.makeLibraryPath dlopenLibraries}
	'';

	# Also a solid solution to fixing the dlopens
	# Using this to package SDL so xsb-static works.
	postInstall = ''
		wrapProgram $out/bin/sbi \
		--prefix LD_LIBRARY_PATH : ${ pkgs.lib.makeLibraryPath [ pkgs.SDL2 ] }
	'';
}
