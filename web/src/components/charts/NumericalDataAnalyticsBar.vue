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
                labelString: "Range Groups",
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
      let labels = Object.keys(data);
      // for (let i = 0; i < labels.length - 1; i++) {
      //   labels[i] = parseFloat(labels[i]).toPrecision(4);
      // }

      // for (const key of labels) {
      //   console.log(parseFloat(key).toPrecision(4))
      //   data[Math.round(parseFloat(key),5)] = data[key];
      //   delete data[key];
      // }

      let arrayOfObj = labels.map(function(d, _) {
        return {
          label: parseFloat(d),
          data: data[d]
        };
      });

      arrayOfObj = arrayOfObj.sort(function(a, b) {
        return a.label < b.label;
      });


      // for (let i = 0; i < labels.length - 1; i++) {
      //   labels[i] = "-" + labels[i+1]
      // }
      // labels[labels.length - 1] += " - " + (2 * (parseInt(labels[labels.length - 1])) - (parseInt(labels[labels.length - 2])))
      let render_data = {
        labels: Object.keys(data),
        datasets: [
          {
            label: this.name,
            backgroundColor: "RGB(99, 255, 222)",
            data: Object.values(data),
          },
        ],
      };
      this.renderChart(render_data, this.options);
    },
  },
};
</script>
