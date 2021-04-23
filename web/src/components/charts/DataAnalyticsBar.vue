<script>
import { Bar } from "vue-chartjs";

export default {
  extends: Bar,
  props: {
    chartData: Object,
    name: String,
    color: String,
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
        labels: Object.keys(data).slice(0, 30),
        datasets: [
          {
            label: this.name,
            backgroundColor: this.color,
            data: Object.values(data).slice(0, 30),
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
