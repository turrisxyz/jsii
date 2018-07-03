﻿using Newtonsoft.Json;
using System;

namespace AWS.Jsii.JsonModel.Api.Request
{
    [JsonObject(MemberSerialization = MemberSerialization.OptIn)]
    public class InvokeRequest : IKernelRequest
    {
        public InvokeRequest(ObjectReference objectReference, string method, object[] arguments = null)
        {
            ObjectReference = objectReference ?? throw new ArgumentNullException(nameof(objectReference));
            Method = method ?? throw new ArgumentNullException(nameof(method));
            Arguments = arguments;
        }

        [JsonProperty("api")]
        public string Api { get; } = "invoke";

        [JsonProperty("objref")]
        public ObjectReference ObjectReference { get; }

        [JsonProperty("method")]
        public string Method { get; }

        [JsonProperty("args", NullValueHandling = NullValueHandling.Ignore)]
        public object[] Arguments { get; }
    }
}