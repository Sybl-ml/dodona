<template>
  <b-container fluid>
    <b-row class="justify-content-center">
      <bar-chart ref="chart" />
    </b-row>
  </b-container>
</template>

<style>
.nodeExpansion {
  width: 100%;
}
</style>

<script>
import BarChart from "@/components/charts/BarChart.vue";

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
    BarChart,
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
