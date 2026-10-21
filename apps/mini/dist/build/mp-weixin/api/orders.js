"use strict";const e=require("./request.js"),r=require("./auth.js");exports.createOrder=function(s){return r.withCustomerSession(()=>e.request("/orders","POST",s))};
