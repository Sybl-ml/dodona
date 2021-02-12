<script>
import { Bar } from "vue-chartjs";

export default {
  extends: Bar,
  data() {
    return {
      options: {
          responsive: true,
          maintainAspectRatio: false,
        },
    }
  },
  methods: {
    renderChartOnCollapse(dataset) {
      let render_data = {
          labels: [1, 2, 3, 4, 5],
          datasets: [
            {
              label: "Performance",
              backgroundColor: "#f87979",
              data: dataset,
            },
          ],
        }
      this.renderChart(render_data, this.options);
    },
  },
};
</script>
