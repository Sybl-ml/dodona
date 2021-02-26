<script>
import { Bar, mixins } from "vue-chartjs";

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
        maintainAspectRatio: false,
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
};
</script>
