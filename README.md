<p align="center">
  <img src="https://raw.githubusercontent.com/onnovalkering/brane/main/contrib/assets/logo.png" alt="logo" width="250"/>
  <h3 align="center">Programmable Orchestration of Applications and Networking</h3>
</p>

----

<span align="center">

  [![Audit status](https://github.com/epi-project/brane/workflows/Audit/badge.svg)](https://github.com/epi-project/brane/actions)
  [![CI status](https://github.com/epi-project/brane/workflows/CI/badge.svg)](https://github.com/epi-project/brane/actions)
  [![License: Apache-2.0](https://img.shields.io/github/license/epi-project/brane.svg)](https://github.com/epi-project/brane/blob/main/LICENSE)
  [![Coverage status](https://coveralls.io/repos/github/epi-project/brane/badge.svg)](https://coveralls.io/github/epi-project/brane)
  [![Release](https://img.shields.io/github/release/epi-project/brane.svg)](https://github.com/epi-project/brane/releases/latest)
  [![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.3890928.svg)](https://doi.org/10.5281/zenodo.3890928)

</span>

---
> # Important Notice
> With the conclusion of the EPI Project, this repository has moved to a new GitHub organisation: [/BraneFramework/brane](https://github.com/BraneFramework/brane). This repository only exists for achival purposes.

---

## Introduction
Regardless of the context and rationale, running distributed applications on geographically dispersed IT resources often comes with various technical and organizational challenges. If not addressed appropriately, these challenges may impede development, and in turn, scientific and business innovation. We have designed and developed Brane to support implementers in addressing these challenges. Brane makes use of containerization to encapsulate functionalities as portable building blocks. Through programmability, application orchestration can be expressed using intuitive domain-specific languages. As a result, end-users with limited or no programming experience are empowered to compose applications by themselves, without having to deal with the underlying technical details.

See the [documentation](https://wiki.enablingpersonalizedinterventions.nl) for more information.


## Installation
For a full how-to on how to install or run the framework, refer to the [user guide](https://wiki.enablingpersonalizedinterventions.nl/user-guide) and then the section that represents your role best. Typically, if you are installing the framework yourself, consult the [installation chapter for administrators](https://wiki.enablingpersonalizedinterventions.nl/user-guide/system-admins/installation/introduction.html); otherwise, consult the [installation chapter for Software Engineers](https://wiki.enablingpersonalizedinterventions.nl/user-guide/software-engineers/installation.html) or [that for Scientists](https://wiki.enablingpersonalizedinterventions.nl/user-guide/scientists/installation.html). The former is a bit more comprehensive, while the latter is a bit more to-the-point and features more visualisations.


## Usage
Similarly to installing it, for using the framework we refer you to the [wiki](https://wiki.enablingpersonalizedinterventions.nl/user-guide). Again, choose the role that suits you best ([System administrators](https://wiki.enablingpersonalizedinterventions.nl/user-guide/system-admins/introduction.html) if you are managing an instance, [Policy experts]() if you are writing policies, [Software engineers](https://wiki.enablingpersonalizedinterventions.nl/user-guide/software-engineers/introduction.html) if you are writing packages or [Scientists](https://wiki.enablingpersonalizedinterventions.nl/user-guide/scientists/introduction.html) if you are writing workflows). You can also follow the chapters in the order presented in the wiki's sidebar for a full overview of everything in the framework.


## Contributing
If you're interrested in contributing, please read the [code of conduct](.github/CODE_OF_CONDUCT.md) and [contributing](.github/CONTRIBUTING.md) guide.

Bug reports and feature requests can be created in the [issue tracker](https://github.com/epi-project/brane/issues).


### Development
If you are intending to develop on the framework, then you should [setup your machine for framework compilation](https://wiki.enablingpersonalizedinterventions.nl/user-guide/system-admins/installation/dependencies.html#compilation-dependencies) (install both the dependencies for runtime and compilation).

Then, you can clone this repository (`git clone https://github.com/epi-project/brane.git`) to some folder of choice, and start working on the source code. You can check the [specification](https://wiki.enablingpersonalizedinterventions.nl/specification/) to learn more about the inner workings of the framework, and remember: `make.py` is your biggest friend when compiling the framework, and `branectl` when running or testing it.

Consult the [code documentation](https://wiki.enablingpersonalizedinterventions.nl/docs/brane/index.html) for more information about the codebase itself. Note, however, that this is generated for the latest release; to consult it for a non-release, navigate to the root of the repository and run:
```bash
cargo doc --release --no-deps --open
```
