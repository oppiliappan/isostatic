
let
  sources = import ./sources.nix;

  rustChannelsOverlay = import "${sources.nixpkgs-mozilla}/rust-overlay.nix";
  rustChannelsSrcOverlay = import "${sources.nixpkgs-mozilla}/rust-src-overlay.nix";

in import sources.nixpkgs {
    overlays = [
      rustChannelsOverlay
      rustChannelsSrcOverlay
      (self: super: {
        rustc = super.latest.rustChannels.stable.rust;
        inherit (super.latest.rustChannels.stable) cargo rust rust-fmt rust-std clippy;
      })
    ];
  }
