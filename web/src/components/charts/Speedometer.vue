<template>
  <div>
    <vue-speedometer
      :value="value()"
      :needleTransitionDuration="4000"
      needleTransition="easeElastic"
      :needleHeightRatio="0.75"
      :maxSegmentLabels="4"
      :segments="20"
      :minValue="0"
      :maxValue="1"
      needleColor="#6391ff"
      :forceRender="true"
      :height="200"
      :width="250"
      currentValueText="Model Performance: ${value}"
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
  props: {
    performance: Array,
  },
  methods: {
    value () {
      if (this.performance.length == 0) {
        return 0
      }
      else {
        return Math.round(((this.performance.reduce(function (total, num){return total + num}))/this.performance.length) * 100) / 100;
      }

    }
  }
};
</script>
