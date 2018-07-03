using AWS.Jsii.Runtime.Deputy;

namespace AWS.Jsii.Tests.Calculator.DerivedClassHasNoProperties
{
    [JsiiClass("jsii-calc", "jsii$jsii_calc$.DerivedClassHasNoProperties.Base", "[]")]
    public class Base : DeputyBase
    {
        public Base(): base(new DeputyProps(new object[]{}))
        {
        }

        protected Base(ByRefValue reference): base(reference)
        {
        }

        protected Base(DeputyProps props): base(props)
        {
        }

        [JsiiProperty("prop", "{\"primitive\":\"string\"}")]
        public virtual string Prop
        {
            get => GetProperty<string>();
            set => SetProperty(value);
        }
    }
}