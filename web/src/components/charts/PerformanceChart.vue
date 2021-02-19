<script>
import { Bar } from "vue-chartjs";

export default {
  extends: Bar,
  data() {
    return {
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          xAxes:[{
            display: true,
            scaleLabel: {
              display: true,
              labelString: "Last 5 Jobs"
            }

          }],
          yAxes: [{
            display: true,
            ticks: {
              suggestedMin: 0.0,
              suggestedMax: 1.0,
            },
          }]
        }
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
              backgroundColor: "#fb8c38",
              data: dataset,
            },
          ],
        }
      this.renderChart(render_data, this.options);
    },
  },
};
</script>
