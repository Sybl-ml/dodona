<template>
  <div>
    <vue-speedometer
      :value="0.4"
      :needleTransitionDuration="4000"
      needleTransition="easeElastic"
      :needleHeightRatio="0.75"
      :maxSegmentLabels="5"
      :segments="20"
      :minValue="0"
      :maxValue="1"
      needleColor="#6391ff"
      :forceRender="true"
    />
  </div>
</template>
<script>
import VueSpeedometer from "vue-speedometer";

export default {
  name: "Speedometer",
  components: {
    VueSpeedometer,
  },
};
</script>
