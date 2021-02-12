<template>
<b-container fluid>
    <b-row class="justify-content-center">
        FUCCKKKKKK
    </b-row>
</b-container>
</template>

<style>
.nodeExpansion {
    width: 100%;
}
</style>

<script>
export default {
    name: "ModelPerformance",
    props: {
        data: Object,
        i: Number,
    },
    mounted (){
        console.log(data)
    },
    computed: {
        status_variant() {
          console.log(this.data._id.$oid)
            if (this.data.status === "NotStarted") {
                return "primary";
            } else if (this.data.status === "Running") {
                return "completed";
            } else if (this.data.status === "Stopped") {
                return "warning";
            } else {
                return "secondary";
            }
        },
    },
};
</script>