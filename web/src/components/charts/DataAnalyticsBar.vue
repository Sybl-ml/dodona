<script>
import { Bar } from "vue-chartjs";

export default {
  extends: Bar,
  props: {
    chartData: Object,
    name: String,
  },
  data() {
    return {
      options: {
        responsive: true,
        maintainAspectRatio: true,
        scales: {
          xAxes: [
            {
              display: true,
              scaleLabel: {
                display: true,
                labelString: "Attribute Values",
              },
            },
          ],
          yAxes: [
            {
              display: true,
              scaleLabel: {
                display: true,
                labelString: "Count",
              },
              ticks: {
                beginAtZero: true,
                stepSize: 1,
              },
            },
          ],
        },
      },
    };
  },
  mounted() {
    this.renderNewData(this.chartData);
  },
  methods: {
    renderNewData(data) {
      let render_data = {
        labels: Object.keys(data),
        datasets: [
          {
            label: this.name,
            backgroundColor: "rgb(255, 99, 132)",
            data: Object.values(data),
          },
        ],
      };
      this.renderChart(render_data, this.options);
    },
  },
  watch: {
    chartData: function() {
      this.renderNewData(this.chartData);
    }
  }
};
</script>
