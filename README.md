<!-- Improved compatibility of back to top link: See: https://gitlab.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]


<!-- PROJECT LOGO -->
<br />
<div align="center">
<h3 align="center">IIO-Niri</h3>

  <p align="center">
  Listen to iio-sensor-proxy and updates Niri output orientation depending on the accelerometer orientation.
    <br />
    <a href="https://github.com/Zhaith-Izaliel/iio-niri?tab=readme-ov-file#usage"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/Zhaith-Izaliel/iio-niri/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/Zhaith-Izaliel/iio-niri/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>


<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

Listen to IIO-Sensor-Proxy and updates Niri output orientation depending on the accelerometer orientation. 

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* [![Rust][Rust]][Rust-url]
* [![Nix][Nix]][Nix-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

IIO-Niri requires the Rust Compiler if you plan to compile it, you will also need Cargo to build the project and DBus dependencies.
* `rustc` ⩾ 1.86.0
* `cargo` ⩾ 1.86.0
* `pkg-config` ⩾ 0.29.2
* `libdbus` ⩾ 1.6

At runtime, the program relies on [IIO-Sensor-Proxy](https://gitlab.freedesktop.org/hadess/iio-sensor-proxy/) to fetch updates on the accelerometer. Make sure it is running alongside IIO-Niri.

If you intend to work with Nix:

* `nix` ⩾ 2.28.4 with [flake support](https://wiki.nixos.org/wiki/Flake).

### Installation

#### From source

1. Clone the repo
   ```sh
   git clone https://github.com/Zhaith-Izaliel/iio-niri.git
   ```
2. Install Cargo and Rustc from your package manager.
3. Build IIO-Niri with Cargo in release mode 
   ```sh
   cargo build --release
   ```
4. An executable for IIO-Niri will be available in `target/release/iio-niri`

#### With Nix

1. Import the project in your flake inputs
   ```nix
   inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    iio-niri = {
      url = "github:Zhaith-Izaliel/iio-niri";
      inputs.nixpkgs.follows = "nixpkgs";
    };
   };
   ```
2. You can then install it from `inputs.iio-niri.packages.${system}.default` where `${system}` is your system descriptor. For Linux, usually `x86_64-linux`.


<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

```
Usage: iio-niri [OPTIONS]

Options:
  -m, --monitor <MONITOR>          The monitor to rotate depending on the accelerometer orientation. Defaults to the first monitor Niri can see
  -t, --timeout <TIMEOUT>          The number of milliseconds before timeout for a dbus request [default: 5000]
  -n, --niri-socket <NIRI_SOCKET>  The path to the niri IPC socket. Can be omitted
  -h, --help                       Print help
  -V, --version                    Print version
```

At runtime, the program relies on [IIO-Sensor-Proxy](https://gitlab.freedesktop.org/hadess/iio-sensor-proxy/) to fetch updates on the accelerometer. Make sure it is running alongside IIO-Niri.

### With NixOS

The provided flake offers a NixOS module to install IIO-Niri as well as an overlay.

Here is a simple example on how to install both the overlay and the module in
your NixOS configuration:

```nix
outputs = { self, nixpkgs, iio-niri }: {
  # replace 'joes-desktop' with your hostname here.
  nixosConfigurations.joes-desktop = nixpkgs.lib.nixosSystem {
    system = "x86_64-linux";
    modules = [
      iio-niri.nixosModules.default
      # ---Snip---
      # Add your own modules here
      # ---Snip---

      # Example to add the overlay to Nixpkgs:
      {
        nixpkgs = {
          overlays = [
            iio-niri.overlays.default
          ];
        };
      }
    ];
  };
};
```

Then enable the module and start IIO-Niri in your Niri configuration.

```nix
{...}: {
  programs.iio-niri.enable = true;
}
```

```kdl
spawn-at-startup "iio-niri" "--monitor" "eDP-1"
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

### How to make a pull request

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/amazing-feature`)
3. Commit your Changes (`git commit -m 'feat: add some amazing-feature'`)
  * Your commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.
4. Push to the Branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request on `develop`

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Top contributors:

<a href="https://github.com/Zhaith-Izaliel/iio-niri/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Zhaith-Izaliel/iio-niri" alt="contrib.rocks image" />
</a>


<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.md` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Zhaith Izaliel - [@zhaith-izaliel.bsky.social](https://bsky.app/profile/zhaith-izaliel.bsky.social)

Project Link: [https://github.com/Zhaith-Izaliel/iio-niri](https://github.com/Zhaith-Izaliel/iio-niri)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* [Best-README-Template](https://github.com/othneildrew/Best-README-Template) for this README
* [IIO-Hyprland](https://github.com/JeanSchoeller/iio-hyprland) for the know how on handling DBus requests and signals for IIO-Sensor-Proxy
* [IIO-Sensor-Proxy](https://gitlab.freedesktop.org/hadess/iio-sensor-proxy/) for the proxy to handle accelerometer requests for this program

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/Zhaith-Izaliel/iio-niri.svg?style=for-the-badge
[contributors-url]: https://github.com/Zhaith-Izaliel/iio-niri/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/Zhaith-Izaliel/iio-niri.svg?style=for-the-badge
[forks-url]: https://github.com/Zhaith-Izaliel/iio-niri/network/members
[stars-shield]: https://img.shields.io/github/stars/Zhaith-Izaliel/iio-niri.svg?style=for-the-badge
[stars-url]: https://github.com/Zhaith-Izaliel/iio-niri/stargazers
[issues-shield]: https://img.shields.io/github/issues/Zhaith-Izaliel/iio-niri.svg?style=for-the-badge
[issues-url]: https://github.com/Zhaith-Izaliel/iio-niri/issues
[license-shield]: https://img.shields.io/github/license/Zhaith-Izaliel/iio-niri.svg?style=for-the-badge
[license-url]: https://github.com/Zhaith-Izaliel/iio-niri/blob/master/LICENSE.md

[Rust]: https://img.shields.io/badge/Rust-B7400F?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://www.rust-lang.org/
[Nix]: https://img.shields.io/badge/nix-0B1120?style=for-the-badge&logo=nixos
[Nix-url]: https://nixos.org/
