<template>
  <b-container fluid>
    <b-row class="justify-content-center">
      <performance-chart ref="chart" />
    </b-row>
  </b-container>
</template>

<style>
.nodeExpansion {
  width: 100%;
}
</style>

<script>
import PerformanceChart from "@/components/charts/PerformanceChart.vue";

export default {
  name: "ModelPerformance",
  props: {
    data: Array,
  },
  data() {
    return {
      performance: [],
      password: "",
      chartLoaded: false,
    };
  },
  components: {
    PerformanceChart,
  },
  methods: {
    show() {
      if (!this.chartLoaded) {
        this.$refs.chart.renderChartOnCollapse(this.data);
        this.chartLoaded = true;
      }
    },
  },
};
</script>
