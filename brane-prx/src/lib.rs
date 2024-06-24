//  LIB.rs
//    by Lut99
//
//  Created:
//    23 Nov 2022, 10:34:23
//  Last edited:
//    23 May 2023, 15:29:39
//  Auto updated?
//    Yes
//
//  Description:
//!   The `brane-prx` service acts as a gateway for outgoing, and sometimes
//!   also incoming, traffic on a node. This is done so that it acts as a
//!   uniform place to do the following two things for every stream:
//!   - Encrypt it using TLS
//!   - Route it through a socksx-proxy.
//!   
//!   The first is nice because BRANE uses X.509 certificates to prove node
//!   identity, meaning that node-to-node communication (or more
//!   specifically, anything going to `brane-reg`) needs to be encrypted.
//!   
//!   The second is nice in the case of integrating Jamila's
//!   [BFC Framework](https://www.enablingpersonalizedinterventions.nl/2022-11-08/rq6-jamilla.pdf),
//!   where we route traffic through virtualized network functions to apply
//!   on-demand security- and network functionality.
//!   
//!   # Features
//!   There are a few specific features for the `brane-prx` service.
//!   
//!   Its first feature is that it can dynamically "forward ports" from the
//!   container network to the outside world. Specifically, using a REST API,
//!   another service can create a new mapping to an external address, at which
//!   point `brane-prx` will allocate a port and open a listener there. Any
//!   incoming connection on this port will be forwarded to the target, while
//!   `brane-prx` applies any of the aforementioned encryption or bridging
//!   functions.
//!
//!   The second feature is that it can "forward" external ports to the
//!   internal part as well, except that these are only statically defined in
//!   the `proxy.yml` file. This is especially useful when the proxy service
//!   is deployed as a standalone proxy node.
//

// Declare modules
pub mod client;
pub mod errors;
pub mod manage;
pub mod ports;
pub mod redirect;
pub mod spec;
