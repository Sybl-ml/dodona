<template>
  <b-container fluid>
    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
        <b-card
          class="mb-4 shadow"
          no-body
          v-b-toggle="'collapse-'+String(this.i)"
          :border-variant="status_variant"
          style="border-width: 0.15rem"
          onfocus="this.blur();"
        >
          <b-row no-gutter>
            <b-col>
              <b-card-body>
                <b-card-title>
                  {{data.name}}
                </b-card-title>
                <b-card-text>
                  <b-icon-clock-fill></b-icon-clock-fill>
                  01:22:30
                </b-card-text>
              </b-card-body>
            </b-col>
            <b-col>
              <b-card-body style="text-align: right">
                <b-card-text v-if="data.status == 'Available'">
                  <b-icon-check-circle-fill small style="color: #00bf26"></b-icon-check-circle-fill>
                  Available
                </b-card-text>
                <b-card-text v-else-if="data.status == 'Disabled'">
                  <b-icon-x-octagon-fill style="color: #ff643d"></b-icon-x-octagon-fill>
                  Disabled
                </b-card-text>
                <b-card-text v-else-if="data.status == 'Locked'">
                  <b-icon-lock-fill style="color: #fbb000"></b-icon-lock-fill>
                  Not Connected
                </b-card-text>
                <b-card-text>
                  <b-icon-cash-stack></b-icon-cash-stack>
                  £2.25
                </b-card-text>
              </b-card-body>
            </b-col>
          </b-row>
          <b-row no-gutter class="justify-content-center">
            <b-icon-chevron-compact-down
              font-scale="1.5"
            ></b-icon-chevron-compact-down>
          </b-row>
        </b-card>
      </b-col>
    </b-row>
    <b-row>
      <b-collapse :id="'collapse-'+String(this.i)" class="mb-4 nodeExpansion">
        <b-card class="shadow">
          <b>API Key</b>
        </b-card>
      </b-collapse>
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
  name: "ModelCard",
  props: {
    data: Object,
    i: Number,
  },
  computed: {
    status_variant(){
      if (this.data.status === "Locked"){
        return "primary"
      }
      else if (this.data.status === "Available"){
        return "completed"
      }
      else if (this.data.status === "Disabled"){
        return "warning"
      }
    }
  },
  methods: {
    
  }
};
</script>
