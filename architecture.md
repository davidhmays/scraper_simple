Direct Mail Attribution Platform

Version: 0.1
Status: Authoritative Domain Contract

1. System Purpose

This system is a Direct Mail Campaign Attribution Platform.

It enables small businesses to:

Design and track marketing campaigns

Attach multiple creative media variants

Send physical mailings

Attribute QR scans back to campaign, media, mailing, and recipient

Analyze demographic performance

The system separates:

Strategic layer → Campaign

Creative layer → Media

Operational layer → Mailing

Data layer → List

Event layer → RecipientInstance + ClickEvent

Campaign is the core abstraction.

2. C4 Model Overview
Level 1 – System Context

Users:

Small business owners

Admin (platform operator)

External Systems:

USPS (physical mail fulfillment)

Print vendors

QR scanning devices (end users scanning mail)

The platform:

Generates QR codes

Tracks scans

Aggregates analytics

Optionally manages list marketplace

Level 2 – Containers

Web Application (Rust + Maud + htmx)

Database (Postgres)

QR Redirect Service (may be same backend)

Analytics Engine (aggregation queries)

Level 3 – Core Domain Components

Campaign Manager

Media Manager

Mailing Manager

List Manager

QR Token Generator

Click Event Processor

Reporting Engine

3. Domain Model
Campaign

Represents a marketing initiative.

Responsibilities:

Owns tracking namespace

Aggregates analytics

Contains media variants

Can exist without mailings

Rules:

A Campaign must exist before Media can be created.

All analytics roll up to Campaign.

Campaign is strategic and non-physical.

Fields (conceptual):

id

user_id

name

slug

status

created_at

Media

Represents a creative variant within a Campaign.

Responsibilities:

Defines creative type (postcard, hanger, etc.)

Generates QR variants

Used inside Mailings

Tracks performance within Campaign

Rules:

Must belong to exactly one Campaign.

Can be used in multiple Mailings.

Does not exist independently of Campaign.

Mailing

Represents a physical send batch.

Responsibilities:

Ties Campaign to a List

Tracks fulfillment status

Generates recipient instances

Rules:

Must belong to exactly one Campaign.

Must reference exactly one List.

Has no analytics logic.

Is operational only.

Statuses:

draft

pending_print

sent

archived

List

Represents a dataset of recipients.

Types:

uploaded (user-provided)

marketplace (platform curated)

system (internal)

Rules:

Contains many ListRows.

May be sold via marketplace.

Mailing references exactly one List.

ListRow

Represents an individual recipient.

Contains:

Address fields

Optional demographic JSON

No analytics logic

RecipientInstance

Represents a single physical mail piece.

Created when:

Mailing is finalized.

Contains:

campaign_id

media_id

mailing_id

list_row_id

qr_token

Rules:

One row per printed piece.

Unique QR token per instance.

This is the atomic attribution unit.

ClickEvent

Represents a QR scan.

Contains:

recipient_instance_id

timestamp

ip_address

user_agent

geo metadata

Rules:

Analytics always derived from ClickEvents.

Never stored redundantly at Campaign level (only aggregated in queries).

4. Data Flow

User creates Campaign.

User creates Media inside Campaign.

User uploads or purchases List.

User creates Mailing referencing Campaign + List.

System generates RecipientInstances for each ListRow.

QR codes contain unique token.

Scan → ClickEvent recorded.

Reporting queries aggregate upward.

Roll-up chain:

ClickEvent → RecipientInstance → Media → Campaign
ClickEvent → RecipientInstance → ListRow (demographics)

5. Invariants (Must Never Be Violated)

Media cannot exist without Campaign.

Mailing cannot exist without Campaign.

Mailing must reference exactly one List.

Campaign owns analytics.

Mailing contains no analytics logic.

RecipientInstance must reference exactly one Media and one ListRow.

ClickEvent must reference exactly one RecipientInstance.

If an AI suggests merging Campaign and Mailing, reject it.

If an AI suggests storing aggregate CTR on Campaign as source-of-truth, reject it.

6. Page Intent
Dashboard

Purpose: High-level campaign performance overview.

Campaign Page

Purpose: Strategic performance center.
Contains:

Aggregated metrics

Media comparisons

Demographic reporting
Allows:

Create Mailing

Add Media

Mailing Page

Purpose: Operational execution.
Contains:

List reference

Media allocations

Fulfillment status
Allows:

Approve for print

Mark sent

Export presort

No analytics computation here.

Media Page

Purpose: Creative management within Campaign.

Lists Page

Purpose: Manage uploaded and marketplace lists.

7. Future Expansion

Planned:

List marketplace monetization

Print fulfillment integration

Presort export system

Multi-channel campaign support (email, digital QR)

API access

Architecture must remain layered:
Strategic (Campaign)
Creative (Media)
Operational (Mailing)
Data (List)
Event (ClickEvent)

8. Primary Design Philosophy

This system separates:

Strategy from Execution.

Campaign is the brain.
Mailing is the body.
Media is the voice.
List is the audience.
ClickEvent is the signal.

End of Architecture Contract
