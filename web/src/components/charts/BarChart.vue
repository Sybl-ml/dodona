<script>
import { Bar } from "vue-chartjs";

export default {
  extends: Bar,
  props: {
    chartdata: {
      type: Object,
      default: null,
    },
    options: {
      type: Object,
      default: null,
    },
  },
  methods: {
    renderChartOnCollapse() {
      this.renderChart(this.chartdata, this.options);
    },
  },
};
</script>
