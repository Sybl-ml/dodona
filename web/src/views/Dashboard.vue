<template>
  <b-container fluid>
    <b-row>
      <b-col xs="12" order-xs="2" lg="3">
        <b-form-input placeholder="Search" block/>
        <b-row v-for="p in projects" :key="p.id">
          <b-col>
          <b-card
          :title="p.name"
          block
          >
          <b-card-text>
              {{p.description}}
            </b-card-text>
          </b-card>
          </b-col>
        </b-row>
      </b-col>
      <b-col>
        <b-tabs>
          <b-tab title="first" active> <br />I'm the first tab </b-tab>
          <b-tab title="second"> <br />I'm the second tab </b-tab>
          <b-tab title="Infinity War"> <br />Trailer tomorrow! </b-tab>
        </b-tabs>
      </b-col>
    </b-row>
  </b-container>
</template>

<script>
export default {
  data() {
    return {
projects: [
      {
        id: 1,
        name: "Project 1",
        description: "Good"
      },
      {
        id: 2,
        name: "Project 2",
        description: "the bad one"
      },
      {
        id: 3,
        name: "Project 3",
        description: "fuck knows"
      },
      {
        id: 4,
        name: "Project 4",
        description: "special project"
      }
    ]
    }
    
  }
}
</script>
