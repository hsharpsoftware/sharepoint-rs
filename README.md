# sharepoint-rs [![Build status](https://ci.appveyor.com/api/projects/status/id1361vojq87g4o6/branch/master?svg=true)](https://ci.appveyor.com/project/davidpodhola/sharepoint-rs/branch/master) [![Build Status](https://travis-ci.org/hsharpsoftware/sharepoint-rs.svg?branch=master)](https://travis-ci.org/hsharpsoftware/sharepoint-rs)
Sharepoint client for Rust mainly for [Sharepoint Online](https://products.office.com/en-us/sharepoint/sharepoint-online-collaboration-software) ([Office365](https://www.office.com/)).

Following the [Sharepoint Online Remote Authentication (and doc upload)](http://paulryan.com.au/2014/spo-remote-authentication-rest/), we can now

- login to Office 365 using user name and password (technical user approach)
- get a list properties using [REST](https://dev.office.com/sharepoint/docs/sp-add-ins/working-with-lists-and-list-items-with-rest)

Now the main efford will be to somehow follow the structure of [PnP JavaScript Core component](https://github.com/SharePoint/PnP-JS-Core)
we [successfully used with F# and Fable before](https://github.com/hsharpsoftware/fable-import-sp-pnp-js).

## Tests
To make the tests working you have to set few environment variables. If setting them in `.profile` make sure all the special characters are saved correctly.

```
export RUST_USERNAME=user@company.onmicrosoft.com
export RUST_PASSWORD=123456
export RUST_LIST_GET_URL="https://company.sharepoint.com/sites/eon/gw_GPP/_api/web/lists/getbytitle('TestColumn')"
export RUST_TITLE=TestColumn
export RUST_SITE=https://company.sharepoint.com/sites/eon/gw_GPP/
```
