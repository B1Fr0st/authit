# Core Principles
1. Licenses. Only focus on auth. No frontend, no webradar, nothing except for license creation, deletion, and authorization.
2. Info. Gather as much data as possible on everything. Good logs, data endpoints for monitoring dashboards, etc.
3. Separate. Business logic and actual database interaction should be in separate functions, all types should be in a separate file / crate, and everything should be organized for maintenance, including splitting into files.
4. Performance. The server should minimize the amount of expensive logic done as a result of an API call, especially done as a result of a public API request.


# API Design

/api/v1
    /public - all functions that should be called without any authorization
-[FIN]       GET      /auth - authorize a license + hwid for a given product
-[FIN]       GET      /product - gets the time remaining & product status for a given product in a license
-[FIN]       GET      /health - heartbeat
    /private - all functions that REQUIRE authorization
-   # note - authorization should be performed using the Authorization: header in https requests
        /license - all license methods
-[FIN]           POST     /generator - generate a new license key with the specified product(s) & time
-[FIN]           PUT      /add-product - add product(s) to the specified license
-[FIN]           PUT      /delete-product - remove product(s) from the specified license
-[WIP]           POST     /ban - ban the specified license, but NOT their HWID
-[WIP]           POST     /unban - unban the specified license, but NOT their hwid
-[WIP]           DELETE   /delete - delete specified license
-[WIP]           PUT      /reset-hwid - reset the HWID associated with the given license
        /hwid - all HWID methods
-[WIP]           POST     /ban - ban a hwid across ALL licenses
-[WIP]           POST     /unban - unban a hwid across ALL licenses
        /product - all product methods
-[FIN]           PUT      /freeze - freezes a given product
-[FIN]           PUT      /unfreeze - unfreezes a given product
-[FIN]           PUT      /create - creates a given product
-[FIN]           DELETE   /delete - deletes a given product
        /data - all monitoring/data endpoints
-[FIN]           GET      /licenses - returns all licenses and their sessions
-[FIN]           GET      /products - returns all current products in the system
-[FIN]           GET      /logins - returns all authorization attempts
-[FIN]           GET      /logs - returns the current logs


# type definitions
License -
    - license_key
    - products
    - HWID
    - sessions
Product:
    - id
    - time
    - started_at
    - frozen
    - frozen_at
Session:
    - started
    - ended: Optional