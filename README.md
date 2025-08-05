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
[![project_license][license-shield]][license-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://gitlab.com/gitlab_namespace/repo_name">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">project_title</h3>

  <p align="center">
    project_description
    <br />
    <a href="https://gitlab.com/gitlab_namespace/repo_name"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://gitlab.com/gitlab_namespace/repo_name">View Demo</a>
    &middot;
    <a href="https://gitlab.com/gitlab_namespace/repo_name/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://gitlab.com/gitlab_namespace/repo_name/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
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

[![Product Name Screen Shot][product-screenshot]](https://example.com)

Here's a blank template to get started. To avoid retyping too much info, do a search and replace with your text editor for the following: `gitlab_namespace`, `repo_name`, `bluesky_handle`, `project_title`, `project_description`, `project_license`, `project_exec`

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* [![Rust][Rust]][Rust-url]
* [![Nix][Nix]][Nix-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

This is an example of how you may give instructions on setting up your project locally.
To get a local copy up and running follow these simple example steps.

### Prerequisites

project_title requires the Rust Compiler if you plan to compile it, you will also need Cargo to build the project.
* `rustc` >= 1.82.0
* `cargo` >= 1.82.0

If you intend to work with Nix:

* `nix` ⩾ 2.24.11 with [flake support](https://wiki.nixos.org/wiki/Flake).

### Installation

#### From source

1. Clone the repo
   ```sh
   git clone https://gitlab.com/gitlab_namespace/repo_name.git
   ```
2. Install Cargo and Rustc from your package manager.
3. Build project_title with Cargo in release mode 
   ```sh
   cargo build --release
   ```
4. An executable for project_title will be available in `target/release/project_exec`

#### With Nix

1. Import the project in your flake inputs
   ```nix
   inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    project_exec = {
      url = "gitlab:gitlab_namespace/repo_name";
      inputs.nixpkgs.follows = "nixpkgs";
    };
   };
   ```
2. You can then install it from `inputs.project_exec.packages.${system}.default` where `${system}` is your system descriptor. For Linux, it is usually `x86_64-linux`.


<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

Use this space to show useful examples of how a project can be used. Additional screenshots, code examples and demos work well in this space. You may also link to more resources.

_For more examples, please refer to the [Documentation](https://example.com)_

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

- [ ] Feature 1
- [ ] Feature 2
- [ ] Feature 3
    - [ ] Nested Feature

See the [open issues](https://gitlab.com/gitlab_namespace/repo_name/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/amazing-feature`)
3. Commit your Changes (`git commit -m 'feat: add some amazing-feature'`)
  * Your commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.
4. Push to the Branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Top contributors:

<a href="https://gitlab.com/gitlab_namespace/gitlab_repo/-/graphs/master?ref_type=heads">
  <img src="https://contrib.rocks/image?repo=gitlab_namespace/repo_name" alt="contrib.rocks image" />
</a>



<!-- LICENSE -->
## License

Distributed under the project_license. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Your Name - [@bluesky_handle](https://bsky.app/profile/bluesky_handle)

Project Link: [https://gitlab.com/gitlab_namespace/repo_name](https://gitlab.com/gitlab_namespace/repo_name)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* [Best-README-Template](https://github.com/othneildrew/Best-README-Template) for this README
* []()
* []()

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/gitlab/contributors/gitlab_namespace/repo_name.svg?style=for-the-badge
[contributors-url]: https://gitlab.com/gitlab_namespace/repo_name/graphs/contributors
[forks-shield]: https://img.shields.io/gitlab/forks/gitlab_namespace/repo_name.svg?style=for-the-badge
[forks-url]: https://gitlab.com/gitlab_namespace/repo_name/network/members
[stars-shield]: https://img.shields.io/gitlab/stars/gitlab_namespace/repo_name.svg?style=for-the-badge
[stars-url]: https://gitlab.com/gitlab_namespace/repo_name/stargazers
[issues-shield]: https://img.shields.io/gitlab/issues/gitlab_namespace/repo_name.svg?style=for-the-badge
[issues-url]: https://gitlab.com/gitlab_namespace/repo_name/issues
[license-shield]: https://img.shields.io/gitlab/license/gitlab_namespace/repo_name.svg?style=for-the-badge
[license-url]: https://gitlab.com/gitlab_namespace/repo_name/blob/master/LICENSE.txt

[product-screenshot]: images/screenshot.png
[Rust]: https://img.shields.io/badge/Rust-B7400F?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://www.rust-lang.org/
[Nix]: https://img.shields.io/badge/nix-0B1120?style=for-the-badge&logo=nixos
[Nix-url]: https://nixos.org/
