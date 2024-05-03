# Hello Cloud

This component generates a webpage on a cloud provider and keeps track of total
visits to the page segemented by region. The counts of regions and visits are
stored in Valkey in this demo, but can be used with any Redis-compatible store,
or anything that satisfied the `wasi:keyvalue` interface.

It currently serves static assets compiled into the component, but could just
as easily serve those assets from a CDN.

See the component's [WIT world](wit/world.wit) for the required links.

## Data Model

* `$region`: the total number of visits from a region. The region itself is
  stored as its region code, which is the short name of the region.
* `$region:os:$os`: the total number of visits from a region on a specific
  operating system.
* `$region:browser:$browser`: the total number of visits from a region from a
  specific browser.
