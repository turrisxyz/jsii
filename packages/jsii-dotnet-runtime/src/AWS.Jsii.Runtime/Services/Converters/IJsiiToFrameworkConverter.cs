﻿using AWS.Jsii.JsonModel.Spec;

namespace AWS.Jsii.Runtime.Services.Converters
{
    public interface IJsiiToFrameworkConverter
    {
        bool TryConvert(TypeReference typeReference, IReferenceMap referenceMap, object value, out object result);
    }
}
