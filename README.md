# sharepoint-rs
Sharepoint client for Rust mainly for [Sharepoint Online](https://products.office.com/en-us/sharepoint/sharepoint-online-collaboration-software) ([Office365](https://www.office.com/)).

Following the [Sharepoint Online Remote Authentication (and doc upload)](http://paulryan.com.au/2014/spo-remote-authentication-rest/), we can now

- login to Office 365 using user name and password (technical user approach)
- get a list properties using [REST](https://dev.office.com/sharepoint/docs/sp-add-ins/working-with-lists-and-list-items-with-rest)

Now the main efford will be done to somehow follow the structure of [PnP JavaScript Core component](https://github.com/SharePoint/PnP-JS-Core)
we [successfully used with F# and Fable before](https://github.com/hsharpsoftware/fable-import-sp-pnp-js).
