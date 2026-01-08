# Core Principles
1. Licenses. Only focus on auth. No frontend, no webradar, nothing except for license creation, deletion, and authorization.
2. Info. Gather as much data as possible on everything. Good logs, data endpoints for monitoring dashboards, etc.
3. Separate. Business logic and actual database interaction should be in separate functions, all types should be in a separate file / crate, and everything should be organized for maintenance, including splitting into files.
4. Performance. The server should minimize the amount of expensive logic done as a result of an API call, especially done as a result of a public API request.



# Brainstorming
Role designations / permission management instead of global API key for private endpoints
    - necessitates at the very minimum management accounts
Implement redis
Use actix-web instead of poem (shit routing impl)
User accounts instead of license keys:
    - Can one user have multiple licenses, or is the account the new license?
    -   Account is the new license, but users can redeem keys to add products to their account if purchased from a reseller
    - link to discord account?
    

# API Design

/api/v1
-[FIN]   GET      /auth - authorize a user for the given product with the given hwid
-[FIN]   GET      /health - heartbeat
        /account - all account methods
-[FIN]       POST     /redeem - redeem a generated key
                      -  user locked
-[WIP]       DELETE   /delete - delete an account
                      -  user locked
                      -  admin locked to delete non-self accounts
-[FIN]       POST     /login - returns a JWT for the web panel and subsequent role operations
-[WIP]       PUT      /add-product - add product(s) to an account
                      -  admin locked
-[WIP]       PUT      /delete-product - remove product(s) from an account
                      -  admin locked
-[WIP]       POST     /ban - ban an account, but NOT their HWID
                      -  support locked
-[WIP]       POST     /unban - unban an account, but NOT their hwid
                      -  support locked
-[WIP]       PUT      /reset-hwid - reset the HWID associated with an account
                      -  support locked
-[FIN]       POST     /set-role - changes the role associated with an account
                      -  admin locked
-[WIP]       POST     /products - lists the products an account owns with the time remaining for each one
                      -  user locked
        /hwid - all HWID methods
-[WIP]       POST     /ban - ban a hwid across ALL accounts
                      -  support locked
-[WIP]       POST     /unban - unban a hwid across ALL accounts
                      -  support locked
        /product - all product methods
-[FIN]       POST     /generate-key - generates a key redeemable for a product for a duration (product time is limited, not the key itself)
                      -  admin/reseller locked
-[FIN]       POST     /compensate - compensates all accounts with extra time for a product
                      -  support locked
-[WIP]       PUT      /freeze - freezes a product
                      -  support locked
-[WIP]       PUT      /unfreeze - unfreezes a product
                      -  support locked
-[WIP]       PUT      /create - creates a product
                      -  admin locked
-[WIP]       DELETE   /delete - deletes a product
                      -  admin locked
        /data - all monitoring/data endpoints
-[WIP]       GET      /licenses - returns all licenses and their login/usage sessions
                      -  admin locked
-[WIP]       GET      /ledger - returns all the logs of actions performed by users with elevated privileges
                      -  admin locked
-[WIP]       GET      /products - returns all current products in the system
                      -  admin locked
-[WIP]       GET      /logins - returns all authorization attempts
                      -  admin locked
-[WIP]       GET      /logs - returns the current logs
                      -  admin locked


# type definitions