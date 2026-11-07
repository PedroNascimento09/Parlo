# Microsoft Store / MSIX distribution path

This document describes one alternative way to reduce Windows warning friction for Parlo: distributing through a channel where Microsoft re-signs the package. It is written as an evaluation, not as an instruction to do it now.

## The basic idea

If Parlo is distributed as an MSIX package through the Microsoft Store, Microsoft re-signs the package as part of that distribution path. In that channel, the installer does not rely on you owning and managing your own code-signing certificate in the same way as a standalone download outside the Store.

That is why this path can matter: it changes who is doing the signing and where the package is delivered.

## What this would mean in practice

If Parlo were distributed this way, the project would likely need to:

- package Parlo as an MSIX-capable app or prepare an MSI/EXE installer intended for Store submission where required;
- create or use a Microsoft Store developer account;
- submit the app for certification and publication;
- adapt the download and update experience around a Store-mediated distribution model rather than a raw installer download from GitHub Releases;
- keep the Store entry and the product documentation in sync.

This is not just “sign the installer once”. It is a different distribution channel with its own rules and maintenance.

## What this does and does not do

This path can reduce the “unsigned, unknown publisher” warning experience for users who install through that channel, because the package is handled through a Store-mediated flow where Microsoft re-signs it.

It does not magically make a raw GitHub release installer stop warning, unless that release is also delivered and signed through the relevant Store/MSIX mechanism.

It also does not change the fact that the project’s functionality, privacy posture, or honesty has to be real. Store certification and distribution have their own rules and review expectations.

## When this makes sense

This path makes more sense if:

- you want distribution through a trusted platform rather than direct downloads outside it;
- you are willing to maintain a Store presence and package format;
- you are comfortable with the Store’s rules, visibility, and update model;
- the app fits the kind of thing the Store is a good fit for.

## When this is less attractive

This path is less attractive if:

- you want the main download to stay a direct installer from GitHub Releases forever;
- you do not want to maintain Store packaging and submission;
- you do not want your distribution to depend on a Store review/approval process;
- the app’s current packaging is not a good fit for MSIX/Store distribution without extra work.

## Honest limits

Even with Store distribution, you still need to be honest about:

- what the app does;
- what it does not do;
- what data stays local and what does not;
- any review or policy status that applies.

A Store presence does not excuse overclaiming, and it does not turn the project into something it is not.

## Relation to other signing routes

This path is one alternative to:

- using a free open-source signing service;
- using your own certificate and signing pipeline;
- continuing to ship unsigned direct downloads with honest documentation and per-file false-positive submission.

They are not the same thing, and they are not interchangeable. Pick one based on where you actually want users to get the app and how much maintenance you want.

## Current status for this project

At the moment, this document is a planning note. It is not a claim that Parlo is on the Store or that it should be moved there now. It exists so the project can compare this route against the others without pretending it is free of cost or effort.
