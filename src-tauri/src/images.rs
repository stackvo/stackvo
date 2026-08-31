//! The ten images this application runs that it did not build.
//!
//! ## The double standard this closes
//!
//! `pkg.rs` carries a rule and states it plainly: `MOVING_TAGS` forbids
//! `latest`, `stable`, `edge`, `main` and `master`, because *"an image that
//! changes under a fixed manifest has no digest the manifest can pin, so it has
//! no place in the chain of trust"*. That rule is enforced against **third-party
//! packages** and against nothing else.
//!
//! Six of the ten below are on `latest`. So the application forbade others from
//! doing exactly what it does itself, and the consequence is the one the rule
//! predicts: the day a broken `cloudflared:latest` is published, every user's
//! tunnels stop working, at once, and there is no version to go back to.
//!
//! ## What is fixed here, and what is deliberately not
//!
//! **Not** the tags. Choosing a pin means naming a version that exists, and
//! this file cannot check that a tag or a digest is real — writing one in would
//! be replacing a known-moving reference with an invented fixed one, which is
//! worse. Picking the pins is a release-time act, against a registry, by
//! somebody who can verify the answer.
//!
//! What is fixed is that **there was nowhere to pin**. The ten values lived in
//! four modules as literals, appeared in no interface, no `.env` and no policy
//! file, and nothing in the application could even say which of them moved. Now
//! they are one table, every one of them is overridable, and the application can
//! report its own moving tags the way it reports everybody else's.
//!
//! ## Why the policy file and not `.env`
//!
//! [`crate::policy::run_image`] is already the single funnel every one of these
//! passes through on its way to `docker run` — one call site per image, added
//! when the registry mirror turned out to be skipping them. It reads the policy
//! and nothing else: no workspace root, no `Env`, no file read per container
//! start. Putting the pins beside the mirror keeps that shape, and it puts them
//! in front of the audience that most wants them — an administrator on a
//! managed machine already has a policy file and already uses it to say where
//! images come from.

use serde::Serialize;

/// One image this application runs but does not build.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Own {
    /// The repository, without a tag — the key a pin is written under, because
    /// a pin means "whenever you would run this repository, run *this*".
    pub repository: &'static str,
    /// What this build ships, tag included.
    pub reference: &'static str,
    /// What it is for, so the list reads as an inventory rather than a dump.
    pub used_for: &'static str,
}

impl Own {
    /// Is the shipped reference on a tag that can change under it?
    ///
    /// The same question `pkg::is_moving_tag` asks of somebody else's package,
    /// asked of ours — which is the whole point of this module existing.
    pub fn moving(&self) -> bool {
        matches!(tag_of(self.reference), Some(tag) if crate::pkg::is_moving_tag(tag))
    }
}

/// Every image this application runs and did not build.
///
/// One table rather than four modules of literals. Before this they were in
/// `tunnel.rs` (seven), `landing.rs`, `tunnelid.rs` and `perf.rs`, and the only
/// way to answer "what does this app pull" was to grep for a colon.
///
/// The tunnel providers keep their own `image` field — it is what the picker
/// and the runner read, and moving it here would make a provider's definition
/// live in two files. This table names them so the inventory is complete, and
/// `provider_images_are_listed_here` holds the two sides equal.
pub const OWN: &[Own] = &[
    Own {
        repository: "cloudflare/cloudflared",
        reference: "cloudflare/cloudflared:latest",
        used_for: "Cloudflare tunnels, named and anonymous",
    },
    Own {
        repository: "ngrok/ngrok",
        reference: "ngrok/ngrok:latest",
        used_for: "ngrok tunnels",
    },
    Own {
        repository: "tailscale/tailscale",
        reference: "tailscale/tailscale:latest",
        used_for: "Tailscale Funnel",
    },
    Own {
        repository: "openziti/zrok",
        reference: "openziti/zrok:latest",
        used_for: "zrok tunnels",
    },
    Own {
        repository: "localxpose/localxpose",
        reference: "localxpose/localxpose:latest",
        used_for: "LocalXpose tunnels",
    },
    Own {
        repository: "kroniak/ssh-client",
        reference: "kroniak/ssh-client:latest",
        used_for: "the SSH reverse tunnel",
    },
    Own {
        repository: "node",
        reference: "node:22-alpine",
        used_for: "localtunnel, which ships as an npm package",
    },
    Own {
        repository: "nginx",
        reference: "nginx:alpine",
        used_for: "the landing page and the tunnel guard",
    },
    Own {
        repository: "alpine",
        reference: "alpine:3",
        used_for: "the performance helper",
    },
];

/// The tag part of a reference, or `None` for a bare repository or a digest.
///
/// The last colon, not the first: `localhost:5000/x:1.2` has two, and only the
/// second is a tag. A `@sha256:…` reference has no tag at all and is already
/// as pinned as a reference gets.
pub fn tag_of(reference: &str) -> Option<&str> {
    if reference.contains('@') {
        return None;
    }
    let (repo, tag) = reference.rsplit_once(':')?;
    // A colon inside the first path component is a registry port, not a tag.
    if tag.contains('/') || repo.is_empty() {
        return None;
    }
    Some(tag)
}

/// The repository part of a reference — everything before the tag or digest.
pub fn repository_of(reference: &str) -> &str {
    if let Some((repo, _)) = reference.split_once('@') {
        return repo;
    }
    match tag_of(reference) {
        Some(tag) => &reference[..reference.len() - tag.len() - 1],
        None => reference,
    }
}

/// The registry host a reference names, when it names one.
///
/// Docker's rule, and it is entirely about the first component: `mysql:8.0` has
/// a colon and no host; `docker.io/library/mysql` has one. A first component
/// counts as a host when it carries a dot or a port, or is `localhost`.
///
/// `None` therefore means **Docker Hub** rather than "no registry" — every
/// unqualified reference is pulled from `docker.io`, and a caller that reported
/// "none" would be reporting the one host it most wanted to name. Callers say
/// so themselves rather than having a default baked in here, because the two
/// readers want different words for it: [`crate::policy`] asks the yes/no
/// question to decide whether to prefix, and [`crate::egress`] wants the name.
pub fn registry_of(reference: &str) -> Option<&str> {
    let (first, _) = reference.split_once('/')?;
    (first.contains('.') || first.contains(':') || first == "localhost").then_some(first)
}

/// What this application ships, and whether it moves — for the screen that
/// shows somebody what their machine will pull.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listed {
    pub repository: String,
    pub used_for: String,
    /// What this build ships.
    pub shipped: String,
    /// What will actually run, after the policy's pin and mirror.
    pub effective: String,
    /// True when the *effective* reference is still on a moving tag. Computed
    /// after the pin, because pinning is exactly what fixes it — a row that
    /// stayed red after somebody pinned it would be a screen that lies.
    pub moving: bool,
    /// Whether a policy pin is what produced `effective`.
    pub pinned: bool,
}

/// The inventory, resolved against the policy in force.
pub fn listed() -> Vec<Listed> {
    OWN.iter()
        .map(|own| {
            let effective = crate::policy::run_image(own.reference);
            Listed {
                repository: own.repository.to_string(),
                used_for: own.used_for.to_string(),
                shipped: own.reference.to_string(),
                moving: matches!(tag_of(&effective), Some(t) if crate::pkg::is_moving_tag(t)),
                pinned: crate::policy::current().image_pin(own.repository).is_some(),
                effective,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reference_splits_into_repository_and_tag() {
        assert_eq!(tag_of("nginx:alpine"), Some("alpine"));
        assert_eq!(repository_of("nginx:alpine"), "nginx");

        assert_eq!(tag_of("cloudflare/cloudflared:latest"), Some("latest"));
        assert_eq!(
            repository_of("cloudflare/cloudflared:latest"),
            "cloudflare/cloudflared"
        );

        // A registry port is a colon that is not a tag.
        assert_eq!(tag_of("localhost:5000/x"), None);
        assert_eq!(repository_of("localhost:5000/x"), "localhost:5000/x");
        assert_eq!(tag_of("localhost:5000/x:1.2"), Some("1.2"));

        // A digest is already pinned and has no tag.
        assert_eq!(tag_of("nginx@sha256:abc"), None);
        assert_eq!(repository_of("nginx@sha256:abc"), "nginx");

        // No tag at all.
        assert_eq!(tag_of("nginx"), None);
        assert_eq!(repository_of("nginx"), "nginx");
    }

    /// The finding, as an assertion: six of the ten ship on a tag this
    /// repository forbids third parties from using.
    ///
    /// A count rather than "some", because the number is what makes it a
    /// double standard rather than an oversight — and because pinning one is
    /// supposed to make this go **down**, which a vaguer test would not notice.
    #[test]
    fn the_moving_tags_this_app_ships_are_counted_rather_than_shrugged_at() {
        let moving: Vec<&str> = OWN
            .iter()
            .filter(|o| o.moving())
            .map(|o| o.repository)
            .collect();

        assert_eq!(
            moving.len(),
            6,
            "the count changed: {moving:?}. Pinning one should make this go \
             down — update the number and say which, in the release note."
        );

        // And the rule being broken is this repository's own, applied to
        // somebody else's package.
        assert!(crate::pkg::is_moving_tag("latest"));
    }

    /// Every tunnel provider's image is in the inventory.
    ///
    /// The providers keep their own `image` field, because it is what the
    /// picker and the runner read and splitting a provider's definition across
    /// two files would be worse. This is what stops the two drifting: a
    /// provider added with an image nothing here lists is an image that pulls
    /// from Docker Hub while a screen says the inventory is complete.
    #[test]
    fn provider_images_are_listed_here() {
        let listed: Vec<&str> = OWN.iter().map(|o| o.repository).collect();

        for provider in crate::tunnel::PROVIDERS {
            let repo = repository_of(provider.image);
            assert!(
                listed.contains(&repo),
                "{} runs {repo}, which the image inventory does not list",
                provider.id
            );
        }
    }
}
