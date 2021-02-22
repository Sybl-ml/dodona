<template>
  <b-container fluid class="d-flex flex-column flex-grow-1">
    <b-row class="justify-content-center text-center">
      <b-col lg="5" md="8" sm="12" xs="12">
        <icon-logo
          class="mt-5 mb-3"
          height="10em"
          width="10em"
          :show_text="false"
        />
        <h1 class="mb-3"><strong>Create A New Account</strong></h1>
        <b-card no-body class="text-left mt-3 mb-5 vh-80">
          <navigatable-tab
            :tabs="[
              { key: '1', title: '1. Name', disabled: false },
              { key: '2', title: '2. Details', disabled: false },
              { key: '3', title: '3. Photo', disabled: true },
              { key: '4', title: '4. Payment', disabled: true },
              { key: '5', title: '5. Create', disabled: false },
            ]"
          >
            <template v-slot:1>
              <b-card-text>To start with what is your name ...</b-card-text>

              <b-form-input
                class="mb-3"
                type="text"
                required
                placeholder="First Name"
                v-model="firstName"
              />
              <b-form-input
                class="mb-3"
                type="text"
                required
                placeholder="Last Name"
                v-model="lastName"
              />
            </template>
            <template v-slot:2>
              <b-card-text>
                Select Your Prefered Currency
              </b-card-text>

              <b-form-select
                class="mb-3"
                v-model="preferedCurrency"
                :options="currencyOptions"
              ></b-form-select>
              <b-card-text>
                Select Your Date of Birth
              </b-card-text>
              <b-form-datepicker v-model="dob" class="mb-3"></b-form-datepicker>
            </template>
            <template v-slot:5>
              <b-card-text
                >Please provide your required login infomation...</b-card-text
              >
              <b-input-group class="mb-3" prepend="@">
                <b-form-input
                  type="email"
                  required
                  placeholder="Enter Email"
                  v-model="email"
                />
              </b-input-group>
              <b-input-group class="mb-3" prepend="#">
                <b-form-input
                  :type="passwordType"
                  required
                  placeholder="Password"
                  v-model="password"
                  pattern="^.{1,32}$"
                  title="Password must contain at least eight characters"
                />
                <template #append>
                  <b-input-group-text>
                    <b-icon
                      style="cursor: pointer;"
                      :icon="passwordIcon"
                      @click="hidePassword = !hidePassword"
                    />
                  </b-input-group-text>
                </template>
              </b-input-group>

              <b-input-group class="mb-3" prepend="#">
                <b-form-input
                  type="password"
                  required
                  id="confirmPass"
                  placeholder="Confirm Password"
                  v-model="confirmPassword"
                /> </b-input-group
              ><b-tooltip
                v-if="confirmPassword != password"
                target="confirmPass"
                triggers="hover"
                variant="warning"
                >Passwords do not match
              </b-tooltip>

              <span id="submitButton" class="float-right">
                <b-button
                  size="sm"
                  variant="ready"
                  type="submit"
                  style="width:10rem"
                  v-b-tooltip.hover
                  @click="onSubmit"
                  :disabled="!validCredentials"
                >
                  SIGN UP
                  <b-spinner v-show="submitted" small></b-spinner>
                  <b-icon-check-all
                    v-show="!submitted && validCredentials"
                  ></b-icon-check-all>
                </b-button> </span
              ><b-tooltip
                v-if="!validCredentials"
                target="submitButton"
                triggers="hover"
                placement="topleft"
                variant="danger"
              >
                Missing or Invalid Credentials
              </b-tooltip>
            </template> 
          </navigatable-tab>
        </b-card>
      </b-col>
    </b-row>
    <b-row class="justify-content-center text-center">
      <b-alert v-model="failed" variant="danger" dismissible>
        Something is wrong with your infomation
      </b-alert>
    </b-row>
  </b-container>
</template>

<script>
import IconLogo from "@/components/icons/IconLogo";
import NavigatableTab from "@/components/NavigatableTab.vue";

export default {
  data() {
    return {
      email: "",
      password: "",
      confirmPassword: "",
      overAge: true,
      preferedCurrency: "",
      dob: "",
      firstName: "",
      lastName: "",

      submitted: false,
      hidePassword: true,
      failed: false,

      currencyOptions: [
        { value: null, text: "Please select an option" },
        { value: "USD", text: "U.S. Dollar (USD)" },
        { value: "GBP", text: "Great British Pound (GBP)" },
        { value: "EUR", text: "Euros (EUR)" },
        { value: "YEN", text: "Japenese Yen (JPY)" },
      ],
    };
  },
  components: {
    IconLogo,
    NavigatableTab,
  },
  computed: {
    validCredentials() {
      return (
        this.email &&
        this.firstName &&
        this.lastName &&
        this.password &&
        this.confirmPassword &&
        this.overAge &&
        this.password === this.confirmPassword
      );
    },
    passwordType() {
      return this.hidePassword ? "password" : "text";
    },
    passwordIcon() {
      return this.hidePassword ? "eye-fill" : "eye-slash-fill";
    },
    invalidFeedback() {
      if (this.firstName.length > 0) {
        return "Enter at least 4 characters.";
      }
      return "Please enter something.";
    },
  },
  methods: {
    sleep(ms) {
      return new Promise((resolve) => {
        setTimeout(resolve, ms);
      });
    },
    async onSubmit() {
      this.submitted = true;
      let response = await this.$http.post("api/users/new", {
        email: this.email,
        password: this.password,
        firstName: this.firstName,
        lastName: this.lastName,
      });

      if (response) {
        response = response.data;
        if (response.token === "null") {
          this.failed = false;
        } else {
          $cookies.set("token", response.token, { path: "/", sameSite: true });
          this.$router.push("dashboard");
        }
      }

      await this.sleep(1000);

      this.failed = true;
      this.submitted = false;
    },
  },
};
</script>
