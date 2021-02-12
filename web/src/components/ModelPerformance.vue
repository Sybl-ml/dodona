<template>
<b-container fluid>
    <b-row class="justify-content-center">
        Model Performance: {{this.data}}
        <line-chart :chartdata="chartdata" :options="options"/>

    </b-row>
</b-container>
</template>

<style>
.nodeExpansion {
    width: 100%;
}
</style>

<script>
import LineChart from '@/components/charts/LineChart.vue'

export default {
    name: "ModelPerformance",
    props: {
        data: Array,
    },
    data() {
        return {
            performance: [],
            password: "",
            chartdata: {
                labels: [1, 2, 3, 4, 5],
                datasets: [{
                            label: 'Performance',
                            backgroundColor: '#f87979',
                            data: []
                        }
                    ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false
            }
        }
    },
    components: {
      LineChart
    },
    mounted() {

        this.chartdata = {
            labels: [1, 2, 3, 4, 5],
            datasets:[{
                            label: 'Performance',
                            backgroundColor: '#f87979',
                            data: this.data

                        }
                    ] 
        }

    }

};
</script>